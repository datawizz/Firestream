const axios = require('axios');

class MockStoreManager {
  constructor(baseUrl = 'http://localhost:3000', apiUrl = 'http://localhost:3001') {
    this.baseUrl = baseUrl;
    this.apiUrl = apiUrl;
  }

  async waitForStore(maxAttempts = 30, interval = 1000) {
    console.log('Waiting for mock store to be ready...');
    
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await axios.get(this.baseUrl, {
          timeout: 5000,
          validateStatus: () => true // Accept any status
        });
        
        if (response.status === 200 && response.data.includes('Example Shop')) {
          console.log('Mock store frontend is ready!');
          // Give React a bit more time to fully mount
          await new Promise(resolve => setTimeout(resolve, 2000));
          return true;
        }
      } catch (error) {
        // Store not ready yet
      }
      
      await new Promise(resolve => setTimeout(resolve, interval));
    }
    
    throw new Error('Mock store failed to start within timeout period');
  }

  async waitForApiServer(maxAttempts = 20, interval = 1000) {
    console.log('Waiting for API server to be ready...');
    
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await axios.get(`${this.apiUrl}/api/health`, {
          timeout: 5000,
          validateStatus: () => true
        });
        
        if (response.status === 200 && response.data.status === 'ok') {
          console.log('API server is ready!');
          return true;
        }
      } catch (error) {
        // API not ready yet
      }
      
      await new Promise(resolve => setTimeout(resolve, interval));
    }
    
    throw new Error('API server failed to start within timeout period');
  }

  async waitForBothServers(maxAttempts = 30, interval = 1000) {
    console.log('Waiting for both frontend and API servers...');
    
    await this.waitForStore(maxAttempts, interval);
    await this.waitForApiServer(maxAttempts, interval);
    
    console.log('Both servers are ready!');
    return true;
  }

  async reset() {
    // Reset the store to initial state
    // This would be implemented as an API endpoint in a real scenario
    console.log('Resetting mock store state...');
    
    // For now, we'll clear session storage by evaluating JS in the browser
    // This will be done through the page object in actual tests
  }

  async setProductCount(count) {
    // In a real implementation, this would be an API call
    // For now, it's a placeholder for future enhancement
    console.log(`Setting product count to ${count}`);
  }

  async setAuthCredentials(username, password) {
    // Store test credentials for validation
    // This would be implemented as an API endpoint
    console.log(`Setting auth credentials for ${username}`);
  }

  async simulateError(errorType) {
    // Simulate different error conditions
    const errorTypes = {
      network: 'Network timeout',
      server: '500 Internal Server Error',
      notFound: '404 Not Found',
      rateLimit: '429 Too Many Requests'
    };
    
    if (!errorTypes[errorType]) {
      throw new Error(`Unknown error type: ${errorType}`);
    }
    
    console.log(`Simulating error: ${errorTypes[errorType]}`);
    // This would be implemented as an API endpoint
  }

  async getProductData(page = 1, perPage = 20) {
    // Get product data for verification
    // This would be an API endpoint in real implementation
    try {
      const response = await axios.get(`${this.baseUrl}/products`, {
        params: { page, perPage }
      });
      return response.data;
    } catch (error) {
      // For now, return mock data
      return {
        products: [],
        total: 100,
        page,
        perPage
      };
    }
  }

  async injectTestHelpers(page) {
    // Inject helper functions into the page for testing
    await page.evaluateOnNewDocument(() => {
      // Capture console logs
      window.__consoleLogs = [];
      const originalLog = console.log;
      const originalError = console.error;
      const originalWarn = console.warn;
      
      console.log = (...args) => {
        window.__consoleLogs.push({ type: 'log', args, timestamp: Date.now() });
        originalLog.apply(console, args);
      };
      
      console.error = (...args) => {
        window.__consoleLogs.push({ type: 'error', args, timestamp: Date.now() });
        originalError.apply(console, args);
      };
      
      console.warn = (...args) => {
        window.__consoleLogs.push({ type: 'warn', args, timestamp: Date.now() });
        originalWarn.apply(console, args);
      };
      
      // Add test helpers
      window.__testHelpers = {
        getProductCount: () => {
          const products = document.querySelectorAll('.product-item');
          return products.length;
        },
        
        getCurrentPage: () => {
          const pageInfo = document.querySelector('.page-info');
          if (pageInfo) {
            const match = pageInfo.textContent.match(/Page (\d+) of (\d+)/);
            return match ? { current: parseInt(match[1]), total: parseInt(match[2]) } : null;
          }
          return null;
        },
        
        isLoggedIn: () => {
          return !!sessionStorage.getItem('user');
        },
        
        getAuthUser: () => {
          const user = sessionStorage.getItem('user');
          return user ? JSON.parse(user) : null;
        },
        
        // Content verification helpers
        waitForContent: (selector, minLength = 1) => {
          const element = document.querySelector(selector);
          if (!element) return false;
          const content = element.textContent || '';
          return content.trim().length >= minLength;
        },
        
        isContentLoaded: (selector = 'body') => {
          const element = document.querySelector(selector);
          if (!element) return false;
          
          // Check for loading indicators
          const hasLoading = element.querySelector('.loading, .spinner, [data-loading="true"]');
          if (hasLoading) return false;
          
          // Check for meaningful content
          const content = element.textContent || '';
          if (content.trim().length < 10) return false;
          
          // Check if images are loaded
          const images = element.querySelectorAll('img');
          for (const img of images) {
            if (!img.complete || img.naturalHeight === 0) return false;
          }
          
          return true;
        },
        
        getContentStats: (selector = 'body') => {
          const element = document.querySelector(selector);
          if (!element) return null;
          
          return {
            textLength: (element.textContent || '').trim().length,
            childCount: element.children.length,
            imageCount: element.querySelectorAll('img').length,
            imagesLoaded: Array.from(element.querySelectorAll('img')).filter(img => img.complete).length,
            hasLoadingIndicators: !!element.querySelector('.loading, .spinner, [data-loading="true"]'),
            visibleText: element.innerText ? element.innerText.trim().substring(0, 100) : ''
          };
        },
        
        isReactMounted: (rootSelector = '#root') => {
          const root = document.querySelector(rootSelector);
          if (!root) return false;
          
          // Check for React fiber
          const hasFiber = root._reactRootContainer || 
                          root.__reactContainer || 
                          (root.firstChild && root.firstChild._reactRootContainer);
          
          // Also check for content
          const hasContent = root.children.length > 0 && 
                           (root.textContent || '').trim().length > 0;
          
          return !!(hasFiber || hasContent);
        }
      };
    });
  }

  async configureNetworkConditions(page, conditions = 'Good 3G') {
    // Configure network throttling for testing
    const presets = {
      'Good 3G': {
        downloadThroughput: 1.5 * 1024 * 1024 / 8,
        uploadThroughput: 750 * 1024 / 8,
        latency: 40
      },
      'Slow 3G': {
        downloadThroughput: 0.5 * 1024 * 1024 / 8,
        uploadThroughput: 0.5 * 1024 * 1024 / 8,
        latency: 2000
      },
      'Offline': {
        offline: true
      }
    };
    
    const config = presets[conditions] || conditions;
    
    if (config.offline) {
      await page.setOfflineMode(true);
    } else {
      const client = await page.target().createCDPSession();
      await client.send('Network.emulateNetworkConditions', {
        downloadThroughput: config.downloadThroughput,
        uploadThroughput: config.uploadThroughput,
        latency: config.latency
      });
    }
    
    console.log(`Network conditions set to: ${conditions}`);
  }

  async getTestCredentials() {
    // Return valid test credentials
    return {
      valid: {
        username: 'test_user',
        password: 'test_pass'
      },
      invalid: {
        username: 'invalid_user',
        password: 'wrong_pass'
      },
      scraper: {
        username: 'scraper_test',
        password: 'scraper_pass'
      }
    };
  }
}

module.exports = MockStoreManager;