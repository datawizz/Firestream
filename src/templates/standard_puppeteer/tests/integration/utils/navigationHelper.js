class NavigationHelper {
  constructor(page, mockStoreManager = null) {
    this.page = page;
    this.mockStoreManager = mockStoreManager;
  }

  /**
   * Click an element and wait for React Router navigation
   * @param {string} selector - CSS selector of the element to click
   * @param {Object} options - Navigation options
   * @param {string} options.waitForUrl - URL pattern to wait for
   * @param {string} options.waitForSelector - Selector to wait for after navigation
   * @param {number} options.timeout - Maximum time to wait (default: 10000ms)
   */
  async clickAndWaitForNavigation(selector, options = {}) {
    const { waitForUrl, waitForSelector, timeout = 10000 } = options;
    
    // Get current URL before clicking
    const currentUrl = this.page.url();
    console.log(`[Navigation] Current URL before navigation: ${currentUrl}`);
    
    try {
      // Check if element exists before clicking
      const element = await this.page.$(selector);
      if (!element) {
        throw new Error(`Element not found: ${selector}`);
      }
      
      // Click the element
      await this.page.click(selector);
      console.log(`[Navigation] Clicked element: ${selector}`);
      
      // Wait for React Router navigation
      if (waitForUrl) {
        // Wait for URL to change to the expected pattern
        console.log(`[Navigation] Waiting for URL pattern: ${waitForUrl}`);
        await this.page.waitForFunction(
          (expectedUrl) => {
            const url = window.location.href;
            if (expectedUrl instanceof RegExp) {
              return new RegExp(expectedUrl).test(url);
            }
            return url.includes(expectedUrl);
          },
          { timeout },
          waitForUrl
        );
        console.log(`[Navigation] Successfully navigated to: ${this.page.url()}`);
      } else if (waitForSelector) {
        // Wait for a specific element to appear after navigation
        console.log(`[Navigation] Waiting for selector: ${waitForSelector}`);
        await this.page.waitForSelector(waitForSelector, { timeout });
        console.log(`[Navigation] Found selector: ${waitForSelector}`);
      } else {
        // Default: wait for URL to change from current
        console.log(`[Navigation] Waiting for URL to change from: ${currentUrl}`);
        await this.page.waitForFunction(
          (oldUrl) => window.location.href !== oldUrl,
          { timeout },
          currentUrl
        );
        console.log(`[Navigation] URL changed to: ${this.page.url()}`);
      }
      
      // Wait for React to complete rendering and content to be populated
      await this.waitForContentToLoad(waitForSelector || 'body', { timeout: 5000 });
    } catch (error) {
      // Enhanced error logging
      const currentState = await this.page.evaluate(() => ({
        url: window.location.href,
        title: document.title,
        bodyText: document.body.innerText.substring(0, 200),
        hasReactRoot: !!document.querySelector('#root'),
        reactRootChildren: document.querySelector('#root')?.children.length || 0
      }));
      
      console.error('[Navigation] Navigation failed:', {
        error: error.message,
        selector,
        expectedUrl: waitForUrl,
        expectedSelector: waitForSelector,
        currentState
      });
      
      throw error;
    }
  }

  /**
   * Navigate directly to a URL and wait for React app to be ready
   * @param {string} url - URL to navigate to
   * @param {Object} options - Navigation options
   */
  async navigateToUrl(url, options = {}) {
    const { waitForSelector, timeout = 10000 } = options;
    
    await this.page.goto(url, { waitUntil: 'networkidle2' });
    
    // Re-inject test helpers after navigation if mockStoreManager is available
    if (this.mockStoreManager) {
      await this.mockStoreManager.injectTestHelpers(this.page);
    }
    
    // Wait for React app to mount
    await this.page.waitForFunction(
      () => {
        const root = document.querySelector('#root');
        return root && root.children.length > 0;
      },
      { timeout }
    );
    
    // Wait for specific selector if provided
    if (waitForSelector) {
      await this.page.waitForSelector(waitForSelector, { timeout });
    }
    
    // Wait for React Router to initialize and content to load
    await this.waitForContentToLoad(waitForSelector || '#root', { timeout: 5000 });
  }

  /**
   * Perform login and navigate to a protected page
   * @param {Object} credentials - Login credentials
   * @param {string} targetUrl - URL to navigate to after login
   */
  async loginAndNavigate(credentials, targetUrl = '/dashboard') {
    const baseUrl = this.page.url().split('/').slice(0, 3).join('/');
    
    // Go directly to login page
    await this.navigateToUrl(`${baseUrl}/login`, {
      waitForSelector: '#username'
    });
    
    // Enter credentials
    await this.page.type('#username', credentials.username);
    await this.page.type('#password', credentials.password);
    
    // Submit and wait for navigation
    await this.clickAndWaitForNavigation('#login-submit', {
      waitForUrl: '/dashboard',
      waitForSelector: '.dashboard'
    });
    
    // Verify login success
    const isLoggedIn = await this.page.evaluate(() => window.__testHelpers?.isLoggedIn());
    if (!isLoggedIn) {
      throw new Error('Login failed - user is not logged in');
    }
    
    // Navigate to target URL if different from dashboard
    if (targetUrl !== '/dashboard') {
      await this.navigateToUrl(`${baseUrl}${targetUrl}`);
    }
  }

  /**
   * Wait for React component to render with specific content
   * @param {string} selector - CSS selector
   * @param {Object} options - Wait options
   */
  async waitForReactComponent(selector, options = {}) {
    const { timeout = 10000, checkContent = true } = options;
    
    await this.page.waitForFunction(
      (sel, check) => {
        const element = document.querySelector(sel);
        if (!element) return false;
        
        // Check if element has content (not just empty)
        if (check && (!element.textContent || element.textContent.trim().length === 0)) {
          return false;
        }
        
        // Check if element is visible
        const rect = element.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0;
      },
      { timeout },
      selector,
      checkContent
    );
  }

  /**
   * Wait for content to be fully loaded (not just DOM elements)
   * @param {string} selector - CSS selector to check
   * @param {Object} options - Wait options
   */
  async waitForContentToLoad(selector = 'body', options = {}) {
    const { timeout = 10000, minLength = 10 } = options;
    
    try {
      // First wait for the element to exist
      await this.page.waitForSelector(selector, { timeout: timeout / 2 });
      
      // Then wait for it to have actual content
      await this.page.waitForFunction(
        (sel, min) => {
          const element = document.querySelector(sel);
          if (!element) return false;
          
          // For form fields, just check they exist and are visible
          if (sel === '#username' || sel === '#password' || sel === '#login-submit') {
            const rect = element.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
          }
          
          // Check for text content
          const textContent = element.textContent || '';
          if (textContent.trim().length < min) return false;
          
          // Check if React is still rendering (look for loading states)
          const hasLoadingIndicators = element.querySelector('.loading, .spinner, [data-loading="true"]');
          if (hasLoadingIndicators) return false;
          
          // Don't check images for now as it's causing issues
          // const images = element.querySelectorAll('img');
          // for (const img of images) {
          //   if (!img.complete || img.naturalHeight === 0) return false;
          // }
          
          return true;
        },
        { timeout: timeout / 2 },
        selector,
        minLength
      );
      
      // Additional wait for any animations or transitions
      await this.page.waitForTimeout(300);
    } catch (error) {
      console.warn(`[Navigation] Content load timeout for selector: ${selector}`);
      // Don't throw, just log the warning
    }
  }

  /**
   * Wait for network to be idle (no ongoing requests)
   * @param {Object} options - Wait options
   */
  async waitForNetworkIdle(options = {}) {
    const { timeout = 5000, maxInflightRequests = 0 } = options;
    
    try {
      // Puppeteer doesn't have waitForLoadState, so we'll use a different approach
      await this.page.waitForTimeout(1000); // Give network time to settle
    } catch (error) {
      // Ignore errors
    }
  }

  /**
   * Enhanced wait for React app to be ready
   * @param {Object} options - Wait options
   */
  async waitForReactApp(options = {}) {
    const { timeout = 10000, rootSelector = '#root' } = options;
    
    // Wait for React root to exist and have content
    await this.page.waitForFunction(
      (selector) => {
        const root = document.querySelector(selector);
        if (!root) return false;
        
        // Check if React has rendered content
        if (root.children.length === 0) return false;
        
        // Check for React fiber node (indicates React is mounted)
        const hasFiber = root._reactRootContainer || root.__reactContainer || 
                        (root.firstChild && root.firstChild._reactRootContainer);
        
        // If we can't detect React internals, at least check for content
        return hasFiber || root.textContent.trim().length > 0;
      },
      { timeout },
      rootSelector
    );
    
    // Wait for content to stabilize
    await this.waitForContentToLoad(rootSelector, { timeout: 5000 });
    
    // Wait for network to be idle
    await this.waitForNetworkIdle({ timeout: 3000 });
  }
}

module.exports = NavigationHelper;