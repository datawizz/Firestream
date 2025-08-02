const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');
const testCredentials = require('../../fixtures/testCredentials.json');

async function cookieApiExtractionTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'cookieApiExtraction',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);

  const results = {
    screenshots: [],
    testResults: [],
    apiResults: {}
  };

  console.log('Starting Cookie-to-API Extraction Test');

  try {
    // Step 1: Navigate to home page
    console.log(`Navigating to: ${mockStoreUrl}`);
    await navigationHelper.navigateToUrl(mockStoreUrl, {
      waitForSelector: '.app-layout'
    });

    // Wait for React app to be ready
    await navigationHelper.waitForReactApp({
      timeout: 15000,
      rootSelector: '#root'
    });

    results.screenshots.push(await screenshotHelper.capture('01_home_page'));

    // Step 2: Perform login to get authenticated cookies
    console.log('Clicking login button...');
    await navigationHelper.waitForReactComponent('.login-button', {
      timeout: 10000,
      checkContent: true
    });

    await navigationHelper.clickAndWaitForNavigation('.login-button', {
      waitForUrl: '/login',
      waitForSelector: '#username'
    });

    // Wait for login form
    await navigationHelper.waitForReactComponent('form', {
      timeout: 10000,
      checkContent: true
    });

    results.screenshots.push(await screenshotHelper.capture('02_login_form'));

    // Enter credentials
    const credentials = testCredentials.valid;
    console.log('Entering credentials...');

    // Clear and enter username
    await page.focus('#username');
    await page.evaluate((val) => {
      const input = document.querySelector('#username');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, credentials.username);

    // Clear and enter password
    await page.focus('#password');
    await page.evaluate((val) => {
      const input = document.querySelector('#password');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, credentials.password);

    results.screenshots.push(await screenshotHelper.capture('03_credentials_entered'));

    // Submit login
    console.log('Submitting login...');
    await page.evaluate(() => {
      const form = document.querySelector('form');
      if (form) {
        const event = new Event('submit', { bubbles: true, cancelable: true });
        form.dispatchEvent(event);
      }
    });

    // Wait for dashboard
    await page.waitForFunction(
      () => window.location.href.includes('/dashboard'),
      { timeout: 5000 }
    );

    await navigationHelper.waitForReactComponent('.dashboard', {
      timeout: 5000,
      checkContent: true
    });

    results.screenshots.push(await screenshotHelper.capture('04_logged_in_dashboard'));

    // Step 3: Extract cookies from browser
    console.log('Extracting cookies from browser...');
    const cookies = await page.cookies();
    
    console.log(`Found ${cookies.length} cookies:`, cookies.map(c => ({
      name: c.name,
      domain: c.domain,
      path: c.path,
      secure: c.secure,
      httpOnly: c.httpOnly
    })));

    // Filter for authentication cookies
    const authCookies = cookies.filter(cookie => 
      ['sessionId', 'authToken'].includes(cookie.name) && 
      cookie.domain.includes('localhost')
    );

    console.log(`Found ${authCookies.length} authentication cookies:`, authCookies);

    if (authCookies.length === 0) {
      throw new Error('No authentication cookies found after login');
    }

    // Synchronize cookies to API domain (port 3001)
    console.log('Synchronizing cookies to API domain...');
    
    // The cookies should already be set with domain=localhost by the frontend
    // Just verify they exist
    const allCookies = await page.cookies();
    console.log('All cookies after login:', allCookies.map(c => ({
      name: c.name,
      domain: c.domain,
      value: c.value.substring(0, 20) + '...'
    })));
    
    // Verify cookies are set for both domains
    const frontendCookies = await page.cookies('http://localhost:3000');
    const apiCookies = await page.cookies('http://localhost:3001');
    console.log(`Frontend cookies: ${frontendCookies.length}, API cookies: ${apiCookies.length}`);

    // Create cookie header for API requests
    const cookieHeader = authCookies
      .map(c => `${c.name}=${c.value}`)
      .join('; ');

    console.log('Cookie header for API requests:', cookieHeader.substring(0, 100) + '...');

    // Step 4: Test API endpoints with cookies
    console.log('Testing API endpoints with extracted cookies...');

    const apiBaseUrl = 'http://localhost:3001';
    const apiEndpoints = [
      { name: 'health', path: '/api/health', requireAuth: false },
      { name: 'auth-status', path: '/api/auth/status', requireAuth: true },
      { name: 'categories', path: '/api/categories', requireAuth: true },
      { name: 'stats', path: '/api/stats', requireAuth: true },
      { name: 'products-page1', path: '/api/products?page=1&limit=10', requireAuth: true },
      { name: 'products-instock', path: '/api/products?inStock=true&limit=5', requireAuth: true }
    ];

    for (const endpoint of apiEndpoints) {
      console.log(`Testing API endpoint: ${endpoint.name}`);
      
      try {
        // Use fetch with credentials instead of manual cookie headers
        const response = await page.evaluate(async (url) => {
          const res = await fetch(url, {
            method: 'GET',
            credentials: 'include',
            headers: {
              'Content-Type': 'application/json'
            }
          });
          
          return {
            status: res.status,
            statusText: res.statusText,
            data: await res.json(),
            headers: Object.fromEntries(res.headers.entries())
          };
        }, `${apiBaseUrl}${endpoint.path}`);

        console.log(`API ${endpoint.name} response:`, {
          status: response.status,
          dataKeys: Object.keys(response.data)
        });

        results.apiResults[endpoint.name] = {
          status: response.status,
          success: response.status >= 200 && response.status < 300,
          data: response.data,
          requireAuth: endpoint.requireAuth
        };

        // Verify authenticated endpoints return expected data
        if (endpoint.requireAuth && response.status === 200) {
          switch (endpoint.name) {
            case 'auth-status':
              if (!response.data.authenticated) {
                throw new Error('Auth status should show authenticated=true');
              }
              break;
            case 'categories':
              if (!response.data.categories || !Array.isArray(response.data.categories)) {
                throw new Error('Categories should return an array');
              }
              break;
            case 'stats':
              if (typeof response.data.total !== 'number') {
                throw new Error('Stats should return numeric total');
              }
              break;
            case 'products-page1':
            case 'products-instock':
              if (!response.data.products || !Array.isArray(response.data.products)) {
                throw new Error('Products should return an array');
              }
              break;
          }
        }

        results.testResults.push({
          testCase: `API ${endpoint.name}`,
          expected: endpoint.requireAuth ? 'authenticated success' : 'public success',
          actual: response.status === 200 ? 'success' : `error ${response.status}`,
          passed: response.status === 200
        });

      } catch (error) {
        console.error(`Error testing ${endpoint.name}:`, error);
        results.testResults.push({
          testCase: `API ${endpoint.name}`,
          error: error.message,
          passed: false
        });
      }
    }

    results.screenshots.push(await screenshotHelper.capture('05_api_tests_completed'));

    // Step 5: Test that cookies work for subsequent requests
    console.log('Testing cookie persistence across multiple API calls...');
    
    const multipleRequests = [];
    for (let i = 0; i < 3; i++) {
      const response = await page.evaluate(async (url) => {
        const res = await fetch(url, {
          method: 'GET',
          credentials: 'include'
        });
        return {
          status: res.status,
          timestamp: new Date().toISOString()
        };
      }, `${apiBaseUrl}/api/auth/status`);
      
      multipleRequests.push(response);
    }

    const allSuccessful = multipleRequests.every(req => req.status === 200);
    console.log('Multiple API calls results:', multipleRequests);

    results.testResults.push({
      testCase: 'Cookie persistence across multiple requests',
      expected: 'all requests successful',
      actual: allSuccessful ? 'all successful' : 'some failed',
      passed: allSuccessful
    });

    // Step 6: Verify API data structure matches expected schema
    const productsResponse = results.apiResults['products-page1'];
    if (productsResponse && productsResponse.success) {
      const products = productsResponse.data.products;
      const firstProduct = products[0];
      
      const requiredFields = ['id', 'name', 'price', 'inStock', 'category'];
      const hasAllFields = requiredFields.every(field => 
        firstProduct.hasOwnProperty(field)
      );

      results.testResults.push({
        testCase: 'API product schema validation',
        expected: 'products have required fields',
        actual: hasAllFields ? 'all fields present' : 'missing fields',
        passed: hasAllFields
      });

      console.log('Sample product data:', firstProduct);
    }

  } catch (error) {
    console.error('Error in cookie API extraction test:', error);
    
    try {
      const errorScreenshot = await screenshotHelper.capture('error_state');
      console.log(`Error screenshot saved: ${errorScreenshot.filepath}`);
    } catch (screenshotError) {
      console.error('Failed to capture error screenshot:', screenshotError);
    }

    results.testResults.push({
      testCase: 'Overall cookie API workflow',
      error: error.message,
      passed: false
    });
  }

  // Check if all tests passed
  const allPassed = results.testResults.every(r => r.passed);
  
  if (!allPassed) {
    const failures = results.testResults.filter(r => !r.passed);
    console.log(`Cookie API extraction tests failed: ${failures.length} out of ${results.testResults.length} tests failed`);
    console.log('Failures:', failures);
  } else {
    console.log('All cookie API extraction tests passed!');
  }

  return {
    success: allPassed,
    screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
    testResults: results.testResults,
    apiResults: results.apiResults,
    totalTests: results.testResults.length,
    passedTests: results.testResults.filter(r => r.passed).length
  };
}

cookieApiExtractionTest.description = 'Tests cookie extraction from browser authentication and subsequent API usage';

module.exports = cookieApiExtractionTest;