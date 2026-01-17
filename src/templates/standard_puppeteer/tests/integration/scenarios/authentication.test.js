const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');
const testCredentials = require('../../fixtures/testCredentials.json');

async function authenticationTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'authentication',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);

  const testCases = [
    {
      name: 'Valid credentials',
      credentials: testCredentials.valid,
      shouldSucceed: true
    },
    {
      name: 'Invalid credentials',
      credentials: testCredentials.invalid,
      shouldSucceed: false
    },
    {
      name: 'Empty credentials',
      credentials: testCredentials.empty,
      shouldSucceed: false
    }
  ];

  const results = {
    screenshots: [],
    testResults: []
  };

  for (const testCase of testCases) {
    console.log(`Testing: ${testCase.name}`);
    
    try {
      // Navigate to home with enhanced waiting
      console.log(`Navigating to: ${mockStoreUrl}`);
      await navigationHelper.navigateToUrl(mockStoreUrl, {
        waitForSelector: '.app-layout'
      });
      console.log(`Current URL: ${page.url()}`);
      
      // Wait for React app to be fully ready
      console.log('Waiting for React app to be fully ready...');
      await navigationHelper.waitForReactApp({
        timeout: 15000,
        rootSelector: '#root'
      });
      console.log('React app is ready');
      
      // Wait for login button to be visible and interactive
      await navigationHelper.waitForReactComponent('.login-button', {
        timeout: 10000,
        checkContent: true
      });
      
      // Debug: check what's on the page
      const pageContent = await page.evaluate(() => {
        return {
          title: document.title,
          body: document.body.innerHTML.substring(0, 500),
          loginButton: !!document.querySelector('.login-button'),
          allButtons: Array.from(document.querySelectorAll('button')).map(b => ({
            className: b.className,
            text: b.textContent
          }))
        };
      });
      console.log('Page content:', JSON.stringify(pageContent, null, 2));
      
      // Click login button and wait for complete navigation
      console.log('Clicking login button and waiting for login form...');
      
      await navigationHelper.clickAndWaitForNavigation('.login-button', {
        waitForUrl: '/login',
        waitForSelector: '#username'
      });
      
      console.log(`After navigation URL: ${page.url()}`);
      
      // Wait for login form to be fully rendered with all fields
      await navigationHelper.waitForReactComponent('form', {
        timeout: 10000,
        checkContent: true
      });
      
      // Ensure all form fields are interactive
      await Promise.all([
        navigationHelper.waitForReactComponent('#username', { checkContent: false }),
        navigationHelper.waitForReactComponent('#password', { checkContent: false }),
        navigationHelper.waitForReactComponent('#login-submit', { checkContent: true })
      ]);
      
      // Clear any existing values
      await page.evaluate(() => {
        const usernameField = document.querySelector('#username');
        const passwordField = document.querySelector('#password');
        if (usernameField) usernameField.value = '';
        if (passwordField) passwordField.value = '';
      });
      
      // Enter credentials with proper React input events
      if (testCase.credentials.username) {
        await page.focus('#username');
        await page.evaluate((val) => {
          const input = document.querySelector('#username');
          const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
          nativeInputValueSetter.call(input, val);
          const event = new Event('input', { bubbles: true });
          input.dispatchEvent(event);
        }, testCase.credentials.username);
      }
      if (testCase.credentials.password) {
        await page.focus('#password');
        await page.evaluate((val) => {
          const input = document.querySelector('#password');
          const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
          nativeInputValueSetter.call(input, val);
          const event = new Event('input', { bubbles: true });
          input.dispatchEvent(event);
        }, testCase.credentials.password);
      }
      
      const screenshotName = `${testCase.name.replace(/\s+/g, '_')}_before_submit`;
      results.screenshots.push(await screenshotHelper.capture(screenshotName));
      
      // Debug: Check form field values before submission
      const formValues = await page.evaluate(() => ({
        username: document.querySelector('#username')?.value,
        password: document.querySelector('#password')?.value
      }));
      console.log('Form values before submit:', formValues);
      console.log('Expected credentials:', testCase.credentials);
      
      // Submit form - try multiple methods
      const submitted = await page.evaluate(() => {
        const form = document.querySelector('form');
        const button = document.querySelector('#login-submit');
        
        if (form) {
          // Try to submit the form directly
          const event = new Event('submit', { bubbles: true, cancelable: true });
          form.dispatchEvent(event);
          return 'form.dispatchEvent';
        } else if (button) {
          // Try clicking the button
          button.click();
          return 'button.click';
        }
        return 'no form or button found';
      });
      console.log('Submit method used:', submitted);
      
      // Wait for form submission to process and page to update
      await navigationHelper.waitForNetworkIdle({ timeout: 3000 });
      
      // Additional wait for React state updates
      await page.waitForTimeout(1000);
      
      // Debug: Check current state after submission
      const postSubmitState = await page.evaluate(() => ({
        url: window.location.href,
        hasError: !!document.querySelector('.error-message'),
        errorText: document.querySelector('.error-message')?.textContent,
        isLoggedIn: !!sessionStorage.getItem('user'),
        user: sessionStorage.getItem('user'),
        sessionStorage: Object.keys(sessionStorage),
        formStillVisible: !!document.querySelector('#username'),
        dashboardVisible: !!document.querySelector('.dashboard'),
        submitButton: document.querySelector('#login-submit')?.outerHTML,
        formElement: !!document.querySelector('form'),
        consoleErrors: (window.__consoleLogs || []).filter(log => log.type === 'error').map(e => e.args)
      }));
      console.log('Post-submit state:', JSON.stringify(postSubmitState, null, 2));
      
      // Wait for either success (dashboard) or error message with content
      if (testCase.shouldSucceed) {
        // For valid credentials, wait for navigation to dashboard
        await page.waitForFunction(
          () => window.location.href.includes('/dashboard'),
          { timeout: 5000 }
        );
        await navigationHelper.waitForReactComponent('.dashboard', {
          timeout: 5000,
          checkContent: true
        });
        // Ensure dashboard content is loaded
        await navigationHelper.waitForContentToLoad('.dashboard', {
          timeout: 5000,
          minLength: 20
        });
      } else {
        // For invalid credentials, wait for error message with content
        await navigationHelper.waitForReactComponent('.error-message', {
          timeout: 5000,
          checkContent: true
        });
      }
      
      // Check result
      const isLoggedIn = await page.evaluate(() => !!sessionStorage.getItem('user'));
      const hasError = await page.$('.error-message');
      const onDashboard = page.url().includes('/dashboard');
      
      const success = testCase.shouldSucceed ? (isLoggedIn && onDashboard) : (!isLoggedIn && hasError);
      
      results.testResults.push({
        testCase: testCase.name,
        expected: testCase.shouldSucceed ? 'success' : 'failure',
        actual: isLoggedIn ? 'success' : 'failure',
        passed: success
      });
      
      const resultScreenshotName = `${testCase.name.replace(/\s+/g, '_')}_result`;
      results.screenshots.push(await screenshotHelper.capture(resultScreenshotName));
      
      // Logout if logged in
      if (isLoggedIn) {
        await page.evaluate(() => {
          sessionStorage.clear();
        });
      }
      
    } catch (error) {
      console.error(`Error in test case ${testCase.name}:`, error);
      // Take screenshot on error
      try {
        const errorScreenshot = await screenshotHelper.capture(`${testCase.name.replace(/\s+/g, '_')}_error`);
        console.log(`Error screenshot saved: ${errorScreenshot.filepath}`);
      } catch (screenshotError) {
        console.error('Failed to capture error screenshot:', screenshotError);
      }
      results.testResults.push({
        testCase: testCase.name,
        error: error.message,
        passed: false
      });
    }
  }

  // Check if all tests passed
  const allPassed = results.testResults.every(r => r.passed);
  
  if (!allPassed) {
    const failures = results.testResults.filter(r => !r.passed);
    throw new Error(`Authentication tests failed: ${failures.length} out of ${testCases.length} tests failed`);
  }

  return {
    success: true,
    screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
    testResults: results.testResults
  };
}

authenticationTest.description = 'Tests various authentication scenarios including valid and invalid credentials';

module.exports = authenticationTest;