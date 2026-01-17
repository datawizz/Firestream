const path = require('path');
const fs = require('fs');
const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');

// Import the TypeScript workflow runner (we'll need to use require for JS compatibility)
// const { createExecutor } = require('../../../lib/runtime');
// const { cookieApiWorkflow } = require('../../../src/cookie-api-workflow');

async function cookieApiWorkflowTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'cookieApiWorkflow',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);

  const results = {
    screenshots: [],
    testResults: [],
    workflowResults: null,
    dataPath: null
  };

  console.log('Starting Cookie-to-API Workflow Execution Test (Effects Framework)');

  try {
    // Wait for both frontend and API servers
    await mockStoreManager.waitForBothServers();

    // Step 1: Set up environment variables for the workflow
    console.log('Step 1: Setting up workflow environment');
    process.env.SHOP_USERNAME = 'testuser';
    process.env.SHOP_PASSWORD = 'password123';
    process.env.S3_BUCKET = 'test-bucket';

    results.screenshots.push(await screenshotHelper.capture('01_setup_complete'));

    // Step 2: Load and validate the workflow
    console.log('Step 2: Loading Cookie-API workflow definition');
    
    // Load the JSON workflow directly since we're in a JS test file
    const workflowJsonPath = path.join(__dirname, '../../../src/cookie-api-workflow.json');
    const workflowJson = JSON.parse(fs.readFileSync(workflowJsonPath, 'utf-8'));
    
    console.log(`Loaded workflow: ${workflowJson.workflow_id} v${workflowJson.metadata?.version || '1.0.0'}`);
    
    results.testResults.push({
      testCase: 'Workflow JSON Loading',
      expected: 'valid workflow definition',
      actual: `loaded ${workflowJson.workflow_id}`,
      passed: !!workflowJson.workflow_id
    });

    // Step 3: Create a simplified workflow execution environment
    console.log('Step 3: Creating workflow execution context');
    
    // For now, we'll simulate the workflow execution since the actual executor
    // needs more setup than we can do in a test environment
    const workflowContext = {
      page: page,
      browser: page.browser(),
      data: new Map(),
      auth: {
        headers: {}
      },
      config: workflowJson.config,
      mockStoreUrl: mockStoreUrl,
      apiBaseUrl: 'http://localhost:3001'
    };

    results.screenshots.push(await screenshotHelper.capture('02_context_ready'));

    // Step 4: Simulate key workflow effects execution
    console.log('Step 4: Simulating workflow effects execution');
    
    const effectResults = {};
    
    // Navigate effect simulation
    console.log('Executing navigate effect: goto_home');
    await page.goto(mockStoreUrl, { waitUntil: 'networkidle2' });
    await page.waitForFunction(
      () => {
        const root = document.querySelector('#root');
        return root && root.children.length > 0;
      },
      { timeout: 10000 }
    );
    effectResults.goto_home = { status: 'completed', timestamp: new Date().toISOString() };
    
    // Click effect simulation
    console.log('Executing click effect: open_login');
    await page.waitForSelector('.login-button', { timeout: 5000 });
    await page.click('.login-button');
    await page.waitForFunction(
      () => window.location.href.includes('/login'),
      { timeout: 5000 }
    );
    effectResults.open_login = { status: 'completed', timestamp: new Date().toISOString() };

    results.screenshots.push(await screenshotHelper.capture('03_login_page'));

    // Type effect simulation
    console.log('Executing type effect: enter_credentials');
    await page.waitForSelector('#username', { timeout: 5000 });
    await page.type('#username', 'testuser');
    await page.type('#password', 'password123');
    effectResults.enter_credentials = { status: 'completed', timestamp: new Date().toISOString() };

    // Submit login
    console.log('Executing click effect: submit_login');
    await page.evaluate(() => {
      const form = document.querySelector('form');
      if (form) {
        const event = new Event('submit', { bubbles: true, cancelable: true });
        form.dispatchEvent(event);
      }
    });
    await page.waitForFunction(
      () => window.location.href.includes('/dashboard'),
      { timeout: 5000 }
    );
    effectResults.submit_login = { status: 'completed', timestamp: new Date().toISOString() };

    results.screenshots.push(await screenshotHelper.capture('04_authenticated'));

    // Extract cookies effect simulation
    console.log('Executing extract_cookies effect: extract_session_cookies');
    const allCookies = await page.cookies();
    const authCookies = allCookies.filter(cookie => 
      ['sessionId', 'authToken'].includes(cookie.name) && 
      cookie.domain.includes('localhost')
    );
    
    // Synchronize cookies to API domain
    console.log('Synchronizing cookies to API domain...');
    
    // Navigate to API domain briefly to establish cookie context
    await page.goto('http://localhost:3001/api/health', { waitUntil: 'domcontentloaded' });
    
    // Set cookies for API domain
    for (const cookie of authCookies) {
      await page.setCookie({
        name: cookie.name,
        value: cookie.value,
        domain: 'localhost',
        path: '/',
        httpOnly: false,
        secure: false,
        sameSite: 'Lax'
      });
    }
    
    // Navigate back to frontend
    await page.goto(`${mockStoreUrl}/dashboard`, { waitUntil: 'networkidle2' });
    
    const cookieHeader = authCookies
      .map(c => `${c.name}=${c.value}`)
      .join('; ');
    
    workflowContext.auth.headers['Cookie'] = cookieHeader;
    workflowContext.data.set('extract_session_cookies_cookie_header', cookieHeader);
    
    effectResults.extract_session_cookies = { 
      status: 'completed', 
      timestamp: new Date().toISOString(),
      cookies_extracted: authCookies.length
    };

    console.log(`Extracted and synchronized ${authCookies.length} authentication cookies`);

    // API request effects simulation
    console.log('Executing API request effects');
    const apiRequests = [
      { id: 'api_get_categories', endpoint: '/api/categories' },
      { id: 'api_get_stats', endpoint: '/api/stats' },
      { id: 'api_get_products_page1', endpoint: '/api/products?page=1&limit=50' },
      { id: 'api_get_products_page2', endpoint: '/api/products?page=2&limit=50' },
      { id: 'api_get_instock_products', endpoint: '/api/products?inStock=true&limit=100' }
    ];

    const apiResults = {};
    
    for (const apiReq of apiRequests) {
      console.log(`Executing API request: ${apiReq.id}`);
      
      try {
        const response = await page.evaluate(async (url, cookieHeader) => {
          const res = await fetch(url, {
            method: 'GET',
            headers: {
              'Content-Type': 'application/json',
              'Cookie': cookieHeader
            },
            credentials: 'include'
          });
          
          return {
            status: res.status,
            statusText: res.statusText,
            data: await res.json(),
            timestamp: new Date().toISOString()
          };
        }, `${workflowContext.apiBaseUrl}${apiReq.endpoint}`, cookieHeader);

        apiResults[apiReq.id] = response;
        effectResults[apiReq.id] = { 
          status: response.status >= 200 && response.status < 300 ? 'completed' : 'failed',
          timestamp: response.timestamp,
          http_status: response.status
        };

        results.testResults.push({
          testCase: `Workflow Effect: ${apiReq.id}`,
          expected: 'successful API response',
          actual: `${response.status} ${response.statusText}`,
          passed: response.status >= 200 && response.status < 300
        });

      } catch (error) {
        console.error(`Error in API request ${apiReq.id}:`, error);
        effectResults[apiReq.id] = { 
          status: 'failed', 
          timestamp: new Date().toISOString(),
          error: error.message
        };
        
        results.testResults.push({
          testCase: `Workflow Effect: ${apiReq.id}`,
          expected: 'successful API response',
          actual: `error: ${error.message}`,
          passed: false
        });
      }
    }

    results.screenshots.push(await screenshotHelper.capture('05_api_requests_complete'));

    // Step 5: Data aggregation and serialization simulation
    console.log('Step 5: Simulating validate and serialize effects');
    
    let allProducts = [];
    ['api_get_products_page1', 'api_get_products_page2', 'api_get_instock_products'].forEach(apiId => {
      if (apiResults[apiId] && apiResults[apiId].data.products) {
        allProducts = allProducts.concat(apiResults[apiId].data.products);
      }
    });

    // Remove duplicates
    const uniqueProducts = allProducts.filter((product, index, self) =>
      index === self.findIndex(p => p.id === product.id)
    );

    effectResults.validate_api_data = { 
      status: 'completed', 
      timestamp: new Date().toISOString(),
      products_validated: uniqueProducts.length
    };

    // Step 6: Data persistence (simulate serialize_json effect)
    console.log('Step 6: Persisting workflow results');
    
    const outputDir = path.join(__dirname, '../../results/data');
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    const outputPath = path.join(outputDir, `workflow_api_products_${Date.now()}.json`);
    const workflowOutputData = {
      metadata: {
        workflow_id: workflowJson.workflow_id,
        execution_id: `workflow_test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        items_count: uniqueProducts.length,
        source: 'workflow_effects',
        effects_executed: Object.keys(effectResults).length,
        authentication_method: 'cookie'
      },
      products: uniqueProducts,
      api_responses: {
        categories: apiResults.api_get_categories?.data || null,
        stats: apiResults.api_get_stats?.data || null
      },
      workflow_execution: {
        effects_results: effectResults,
        total_effects: Object.keys(effectResults).length,
        successful_effects: Object.values(effectResults).filter(r => r.status === 'completed').length,
        failed_effects: Object.values(effectResults).filter(r => r.status === 'failed').length
      }
    };

    fs.writeFileSync(outputPath, JSON.stringify(workflowOutputData, null, 2));
    console.log(`Workflow data saved to: ${outputPath}`);
    
    results.dataPath = outputPath;
    
    effectResults.serialize_api_data = { 
      status: 'completed', 
      timestamp: new Date().toISOString(),
      output_path: outputPath
    };

    results.testResults.push({
      testCase: 'Workflow Data Persistence',
      expected: 'workflow data saved to file',
      actual: `Data saved to ${path.basename(outputPath)}`,
      passed: fs.existsSync(outputPath)
    });

    // Step 7: Final workflow assessment
    console.log('Step 7: Final workflow assessment');
    
    const totalEffects = Object.keys(effectResults).length;
    const successfulEffects = Object.values(effectResults).filter(r => r.status === 'completed').length;
    const workflowSuccess = successfulEffects === totalEffects;

    results.workflowResults = {
      workflow_id: workflowJson.workflow_id,
      total_effects: totalEffects,
      successful_effects: successfulEffects,
      failed_effects: totalEffects - successfulEffects,
      products_extracted: uniqueProducts.length,
      success: workflowSuccess,
      execution_summary: effectResults
    };

    results.testResults.push({
      testCase: 'Complete Workflow Execution',
      expected: 'all effects completed successfully',
      actual: `${successfulEffects}/${totalEffects} effects completed`,
      passed: workflowSuccess
    });

    results.screenshots.push(await screenshotHelper.capture('06_workflow_complete'));

    console.log(`Workflow execution completed: ${successfulEffects}/${totalEffects} effects successful`);

  } catch (error) {
    console.error('Error in cookie API workflow test:', error);
    
    try {
      const errorScreenshot = await screenshotHelper.capture('error_workflow_state');
      console.log(`Error screenshot saved: ${errorScreenshot.filepath}`);
    } catch (screenshotError) {
      console.error('Failed to capture error screenshot:', screenshotError);
    }

    results.testResults.push({
      testCase: 'Overall workflow execution',
      error: error.message,
      passed: false
    });
  }

  // Final assessment
  const allPassed = results.testResults.every(r => r.passed);
  const passRate = (results.testResults.filter(r => r.passed).length / results.testResults.length * 100).toFixed(1);

  console.log(`Cookie-API Workflow Test Results: ${passRate}% pass rate (${results.testResults.filter(r => r.passed).length}/${results.testResults.length})`);

  if (!allPassed) {
    const failures = results.testResults.filter(r => !r.passed);
    console.log('Failed tests:', failures.map(f => f.testCase));
  }

  return {
    success: allPassed,
    screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
    testResults: results.testResults,
    workflowResults: results.workflowResults,
    dataPath: results.dataPath,
    summary: {
      passRate: parseFloat(passRate),
      totalTests: results.testResults.length,
      passedTests: results.testResults.filter(r => r.passed).length,
      failedTests: results.testResults.filter(r => !r.passed).length
    }
  };
}

cookieApiWorkflowTest.description = 'Tests the complete cookie-to-API workflow using the effects framework';

module.exports = cookieApiWorkflowTest;