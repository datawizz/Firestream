const path = require('path');
const fs = require('fs');
const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');
const testCredentials = require('../../fixtures/testCredentials.json');

async function fullCookieWorkflowTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'fullCookieWorkflow',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);

  const results = {
    screenshots: [],
    testResults: [],
    cookieData: null,
    apiData: {},
    workflowSteps: []
  };

  let outputPath = null;

  console.log('Starting Full Cookie-to-API Workflow Test');

  try {
    // Wait for both frontend and API servers
    await mockStoreManager.waitForBothServers();

    // Step 1: Navigate and authenticate (similar to DOM workflow)
    console.log('Step 1: Authentication via DOM');
    results.workflowSteps.push('authentication_start');

    await navigationHelper.navigateToUrl(mockStoreUrl, {
      waitForSelector: '.app-layout'
    });

    await navigationHelper.waitForReactApp({
      timeout: 15000,
      rootSelector: '#root'
    });

    results.screenshots.push(await screenshotHelper.capture('01_initial_page'));

    // Perform login
    await navigationHelper.waitForReactComponent('.login-button', {
      timeout: 10000,
      checkContent: true
    });

    await navigationHelper.clickAndWaitForNavigation('.login-button', {
      waitForUrl: '/login',
      waitForSelector: '#username'
    });

    // Enter credentials
    const credentials = testCredentials.valid;
    await page.focus('#username');
    await page.evaluate((val) => {
      const input = document.querySelector('#username');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, credentials.username);

    await page.focus('#password');
    await page.evaluate((val) => {
      const input = document.querySelector('#password');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, credentials.password);

    results.screenshots.push(await screenshotHelper.capture('02_credentials_filled'));

    await page.evaluate(() => {
      const form = document.querySelector('form');
      if (form) {
        const event = new Event('submit', { bubbles: true, cancelable: true });
        form.dispatchEvent(event);
      }
    });

    // Wait for successful login
    await page.waitForFunction(
      () => window.location.href.includes('/dashboard'),
      { timeout: 5000 }
    );

    await navigationHelper.waitForReactComponent('.dashboard', {
      timeout: 5000,
      checkContent: true
    });

    results.screenshots.push(await screenshotHelper.capture('03_authenticated_dashboard'));
    results.workflowSteps.push('authentication_complete');

    results.testResults.push({
      testCase: 'DOM Authentication',
      expected: 'successful login to dashboard',
      actual: 'logged in successfully',
      passed: true
    });

    // Step 2: Extract cookies (mimicking extract_cookies effect)
    console.log('Step 2: Cookie Extraction');
    results.workflowSteps.push('cookie_extraction_start');

    const allCookies = await page.cookies();
    const authCookies = allCookies.filter(cookie => 
      ['sessionId', 'authToken'].includes(cookie.name) && 
      cookie.domain.includes('localhost')
    );

    results.cookieData = {
      totalCookies: allCookies.length,
      authCookies: authCookies.length,
      cookieDetails: authCookies.map(c => ({
        name: c.name,
        domain: c.domain,
        path: c.path,
        secure: c.secure,
        httpOnly: c.httpOnly,
        hasValue: !!c.value
      }))
    };

    console.log('Extracted cookie data:', results.cookieData);
    results.workflowSteps.push('cookie_extraction_complete');

    results.testResults.push({
      testCase: 'Cookie Extraction',
      expected: 'authentication cookies found',
      actual: `${authCookies.length} auth cookies extracted`,
      passed: authCookies.length > 0
    });

    if (authCookies.length === 0) {
      throw new Error('No authentication cookies found - cannot proceed with API calls');
    }

    // Synchronize cookies to API domain (port 3001)
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
    
    // Verify cookies are set for both domains
    const frontendCookies = await page.cookies('http://localhost:3000');
    const apiCookies = await page.cookies('http://localhost:3001');
    console.log(`Frontend cookies: ${frontendCookies.length}, API cookies: ${apiCookies.length}`);

    // Create cookie header for API requests
    const cookieHeader = authCookies
      .map(c => `${c.name}=${c.value}`)
      .join('; ');

    // Step 3: API calls with cookies (mimicking api_request effects)
    console.log('Step 3: API Requests with Cookies');
    results.workflowSteps.push('api_requests_start');

    const apiBaseUrl = 'http://localhost:3001';
    const apiSequence = [
      { name: 'categories', path: '/api/categories' },
      { name: 'stats', path: '/api/stats' },
      { name: 'products_page1', path: '/api/products?page=1&limit=20' },
      { name: 'products_page2', path: '/api/products?page=2&limit=20' },
      { name: 'instock_products', path: '/api/products?inStock=true&limit=50' }
    ];

    for (const api of apiSequence) {
      console.log(`Making API request: ${api.name}`);
      
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
        }, `${apiBaseUrl}${api.path}`, cookieHeader);

        results.apiData[api.name] = response;

        const success = response.status >= 200 && response.status < 300;
        console.log(`API ${api.name}: ${response.status} ${response.statusText}`);

        results.testResults.push({
          testCase: `API Request: ${api.name}`,
          expected: 'successful authenticated response',
          actual: `${response.status} ${response.statusText}`,
          passed: success
        });

        // Validate specific response structures
        if (success) {
          switch (api.name) {
            case 'categories':
              const hasCategories = response.data.categories && Array.isArray(response.data.categories);
              results.testResults.push({
                testCase: `API ${api.name} structure`,
                expected: 'categories array',
                actual: hasCategories ? 'categories array found' : 'invalid structure',
                passed: hasCategories
              });
              break;

            case 'stats':
              const hasStats = typeof response.data.total === 'number';
              results.testResults.push({
                testCase: `API ${api.name} structure`,
                expected: 'stats with total',
                actual: hasStats ? 'valid stats' : 'invalid structure',
                passed: hasStats
              });
              break;

            case 'products_page1':
            case 'products_page2':
            case 'instock_products':
              const hasProducts = response.data.products && Array.isArray(response.data.products);
              const hasProductCount = hasProducts && response.data.products.length > 0;
              results.testResults.push({
                testCase: `API ${api.name} structure`,
                expected: 'products array with data',
                actual: hasProductCount ? `${response.data.products.length} products` : 'no products',
                passed: hasProductCount
              });

              // Validate product schema for first product
              if (hasProductCount) {
                const product = response.data.products[0];
                const requiredFields = ['id', 'name', 'price', 'inStock', 'category'];
                const hasAllFields = requiredFields.every(field => 
                  product.hasOwnProperty(field)
                );
                results.testResults.push({
                  testCase: `API ${api.name} product schema`,
                  expected: 'products have required fields',
                  actual: hasAllFields ? 'schema valid' : 'missing fields',
                  passed: hasAllFields
                });
              }
              break;
          }
        }

      } catch (error) {
        console.error(`Error in API request ${api.name}:`, error);
        results.testResults.push({
          testCase: `API Request: ${api.name}`,
          error: error.message,
          passed: false
        });
      }
    }

    results.workflowSteps.push('api_requests_complete');

    // Step 4: Data validation and aggregation (mimicking validate effect)
    console.log('Step 4: Data Validation and Aggregation');
    results.workflowSteps.push('data_validation_start');

    let totalProducts = 0;
    let allProducts = [];

    ['products_page1', 'products_page2', 'instock_products'].forEach(apiName => {
      if (results.apiData[apiName] && results.apiData[apiName].data.products) {
        const products = results.apiData[apiName].data.products;
        totalProducts += products.length;
        allProducts = allProducts.concat(products);
      }
    });

    // Remove duplicates based on product ID
    const uniqueProducts = allProducts.filter((product, index, self) =>
      index === self.findIndex(p => p.id === product.id)
    );

    const validationResults = {
      totalApiProducts: totalProducts,
      uniqueProducts: uniqueProducts.length,
      duplicates: totalProducts - uniqueProducts.length,
      categories: [...new Set(uniqueProducts.map(p => p.category))],
      inStockCount: uniqueProducts.filter(p => p.inStock).length,
      priceRange: {
        min: Math.min(...uniqueProducts.map(p => p.price)),
        max: Math.max(...uniqueProducts.map(p => p.price)),
        avg: uniqueProducts.reduce((sum, p) => sum + p.price, 0) / uniqueProducts.length
      }
    };

    console.log('Data validation results:', validationResults);
    results.workflowSteps.push('data_validation_complete');

    results.testResults.push({
      testCase: 'Data Aggregation',
      expected: 'products aggregated successfully',
      actual: `${validationResults.uniqueProducts} unique products from ${validationResults.totalApiProducts} total`,
      passed: validationResults.uniqueProducts > 0
    });

    // Step 4.5: Save API data to disk (like DOM workflow does)
    console.log('Step 4.5: Saving API data to disk');
    results.workflowSteps.push('data_persistence_start');

    const outputDir = path.join(__dirname, '../../results/data');
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    outputPath = path.join(outputDir, `api_products_${Date.now()}.json`);
    const outputData = {
      metadata: {
        workflow_id: 'cookie_api_scraper',
        execution_id: `api_test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        items_count: uniqueProducts.length,
        source: 'api',
        api_endpoints_called: Object.keys(results.apiData).length,
        authentication_method: 'cookie'
      },
      products: uniqueProducts,
      api_responses: {
        categories: results.apiData.categories?.data || null,
        stats: results.apiData.stats?.data || null
      },
      validation_results: validationResults
    };

    fs.writeFileSync(outputPath, JSON.stringify(outputData, null, 2));
    console.log(`API data saved to: ${outputPath}`);
    results.workflowSteps.push('data_persistence_complete');

    results.testResults.push({
      testCase: 'API Data Persistence',
      expected: 'API data saved to file',
      actual: `Data saved to ${path.basename(outputPath)}`,
      passed: fs.existsSync(outputPath)
    });

    // Step 5: Final workflow validation
    console.log('Step 5: Final Workflow Validation');
    results.workflowSteps.push('workflow_completion');

    const criticalSteps = [
      'authentication_complete',
      'cookie_extraction_complete', 
      'api_requests_complete',
      'data_validation_complete',
      'data_persistence_complete'
    ];

    const workflowComplete = criticalSteps.every(step => 
      results.workflowSteps.includes(step)
    );

    results.testResults.push({
      testCase: 'Complete Cookie-to-API Workflow',
      expected: 'all workflow steps completed',
      actual: workflowComplete ? 'workflow completed successfully' : 'workflow incomplete',
      passed: workflowComplete
    });

    results.screenshots.push(await screenshotHelper.capture('04_workflow_complete'));

    // Summary
    const summary = {
      workflowSteps: results.workflowSteps.length,
      totalTests: results.testResults.length,
      passedTests: results.testResults.filter(r => r.passed).length,
      cookiesExtracted: results.cookieData.authCookies,
      apiCallsSuccessful: Object.values(results.apiData).filter(api => 
        api.status >= 200 && api.status < 300
      ).length,
      uniqueProductsFound: validationResults.uniqueProducts
    };

    console.log('Workflow Summary:', summary);

  } catch (error) {
    console.error('Error in full cookie workflow test:', error);
    
    try {
      const errorScreenshot = await screenshotHelper.capture('error_final_state');
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

  console.log(`Cookie-to-API Workflow Test Results: ${passRate}% pass rate (${results.testResults.filter(r => r.passed).length}/${results.testResults.length})`);

  if (!allPassed) {
    const failures = results.testResults.filter(r => !r.passed);
    console.log('Failed tests:', failures.map(f => f.testCase));
  }

  return {
    success: allPassed,
    screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
    testResults: results.testResults,
    workflowSteps: results.workflowSteps,
    cookieData: results.cookieData,
    apiData: results.apiData,
    dataPath: outputPath || null,
    summary: {
      passRate: parseFloat(passRate),
      totalTests: results.testResults.length,
      passedTests: results.testResults.filter(r => r.passed).length,
      failedTests: results.testResults.filter(r => !r.passed).length
    }
  };
}

fullCookieWorkflowTest.description = 'Complete end-to-end test of cookie-based authentication and API data extraction workflow';

module.exports = fullCookieWorkflowTest;