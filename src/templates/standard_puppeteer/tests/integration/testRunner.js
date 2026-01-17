const puppeteer = require('puppeteer');
const path = require('path');
const TestSetup = require('./setup');
const TestTeardown = require('./teardown');
const MockStoreManager = require('./utils/mockStoreManager');

class TestRunner {
  constructor(config = {}) {
    this.config = {
      headless: process.env.HEADLESS !== 'false',
      slowMo: parseInt(process.env.SLOW_MO) || 0,
      devtools: process.env.DEVTOOLS === 'true',
      mockStorePort: parseInt(process.env.MOCK_STORE_PORT) || 3000,
      apiPort: parseInt(process.env.API_PORT) || 3001,
      scraperPath: path.join(__dirname, '../../'),
      timeout: 30000,
      ...config
    };
    
    this.setup = new TestSetup({
      mockStorePort: this.config.mockStorePort,
      apiPort: this.config.apiPort
    });
    this.teardown = new TestTeardown();
    this.browser = null;
    this.page = null;
    this.results = {
      passed: 0,
      failed: 0,
      tests: [],
      duration: 0,
      startTime: null,
      endTime: null
    };
  }

  async initialize() {
    try {
      console.log('Initializing test environment...');
      
      // Start both mock store and API server
      const { mockStoreUrl, apiUrl } = await this.setup.setup();
      this.mockStoreUrl = mockStoreUrl;
      this.apiUrl = apiUrl;
      
      // Initialize mock store manager with both URLs
      this.mockStoreManager = new MockStoreManager(mockStoreUrl, apiUrl);
      await this.mockStoreManager.waitForBothServers();
      
      // Launch browser
      const executablePath = process.env.PUPPETEER_EXECUTABLE_PATH;
      
      if (executablePath) {
        console.log(`Using system Chromium at: ${executablePath}`);
      } else {
        console.log('Using Puppeteer bundled Chromium');
      }
      
      this.browser = await puppeteer.launch({
        headless: 'new', // Use new headless mode
        slowMo: this.config.slowMo,
        devtools: this.config.devtools,
        ...(executablePath && { executablePath }),
        args: [
          '--no-sandbox',
          '--disable-setuid-sandbox',
          '--disable-dev-shm-usage',
          '--disable-accelerated-2d-canvas',
          '--no-first-run',
          '--no-zygote',
          '--disable-gpu',
          '--disable-gpu-sandbox',
          '--font-render-hinting=none', // Better font rendering in headless
          '--disable-font-subpixel-positioning', // Consistent font rendering
          '--disable-web-security', // Allow cross-origin requests
          '--disable-features=IsolateOrigins,site-per-process', // Better iframe handling
          '--disable-site-isolation-trials', // Better iframe handling
          '--window-size=1920,1080', // Set initial window size
          '--force-device-scale-factor=1', // Consistent scaling
          '--force-color-profile=srgb', // Consistent color rendering
          '--disable-blink-features=AutomationControlled', // Make browser appear more natural
          '--user-agent=Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' // Standard user agent
        ],
        defaultViewport: {
          width: 1920,
          height: 1080,
          deviceScaleFactor: 1,
          hasTouch: false,
          isLandscape: true,
          isMobile: false
        },
        timeout: 60000 // Increase browser launch timeout
      });
      
      // Create page
      this.page = await this.browser.newPage();
      
      // Set additional page configurations for better rendering
      await this.page.setExtraHTTPHeaders({
        'Accept-Language': 'en-US,en;q=0.9'
      });
      
      // Track console logs for debugging
      await this.page.evaluateOnNewDocument(() => {
        window.__consoleLogs = [];
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;
        
        console.log = function(...args) {
          window.__consoleLogs.push({ type: 'log', message: args.join(' '), timestamp: Date.now() });
          originalLog.apply(console, args);
        };
        
        console.error = function(...args) {
          window.__consoleLogs.push({ type: 'error', message: args.join(' '), timestamp: Date.now() });
          originalError.apply(console, args);
        };
        
        console.warn = function(...args) {
          window.__consoleLogs.push({ type: 'warn', message: args.join(' '), timestamp: Date.now() });
          originalWarn.apply(console, args);
        };
      });
      
      // Monitor console logs with more detail
      this.page.on('console', msg => {
        const type = msg.type();
        if (type === 'error' || type === 'warning') {
          console.error(`Browser console ${type}:`, msg.text());
          // Log the full error object for debugging
          msg.args().forEach(async (arg, i) => {
            try {
              const val = await arg.jsonValue();
              if (val && typeof val === 'object') {
                console.error(`  Arg[${i}]:`, JSON.stringify(val, null, 2));
              }
            } catch (e) {
              // Ignore serialization errors
            }
          });
        }
      });
      
      // Monitor page crashes
      this.page.on('pageerror', error => {
        console.error('Page error:', error.message);
      });
      
      // Inject test helpers
      await this.mockStoreManager.injectTestHelpers(this.page);
      
      // Set up request interception for debugging
      if (this.config.devtools) {
        await this.page.setRequestInterception(true);
        this.page.on('request', request => {
          console.log('Request:', request.method(), request.url());
          request.continue();
        });
      }
      
      console.log('Test environment initialized successfully');
      return true;
    } catch (error) {
      console.error('Failed to initialize test environment:', error);
      throw error;
    }
  }

  async runTest(testName, testFunction, options = {}) {
    const testResult = {
      name: testName,
      status: 'pending',
      duration: 0,
      error: null,
      screenshots: [],
      startTime: Date.now()
    };
    
    console.log(`\nRunning test: ${testName}`);
    
    try {
      // Create new page for each test
      const testPage = await this.browser.newPage();
      await testPage.setViewport({ width: 1920, height: 1080 });
      
      
      await this.mockStoreManager.injectTestHelpers(testPage);
      
      // Run the test
      const result = await testFunction(testPage, {
        mockStoreUrl: this.mockStoreUrl,
        apiUrl: this.apiUrl,
        mockStoreManager: this.mockStoreManager,
        ...options
      });
      
      // Collect screenshots if any
      if (result && result.screenshots) {
        testResult.screenshots = result.screenshots;
      }
      
      testResult.status = 'passed';
      this.results.passed++;
      console.log(`✓ ${testName} passed`);
      
      // Close test page
      await testPage.close();
    } catch (error) {
      testResult.status = 'failed';
      testResult.error = error.message;
      this.results.failed++;
      console.error(`✗ ${testName} failed:`, error.message);
      
      // Take error screenshot if page exists
      if (this.page && !this.page.isClosed()) {
        try {
          const errorScreenshot = path.join(__dirname, '../results/screenshots', `${testName}_error_${Date.now()}.png`);
          await this.page.screenshot({ path: errorScreenshot, fullPage: true });
          testResult.screenshots.push(errorScreenshot);
        } catch (e) {
          // Ignore screenshot errors
        }
      }
    } finally {
      testResult.endTime = Date.now();
      testResult.duration = testResult.endTime - testResult.startTime;
      this.results.tests.push(testResult);
    }
    
    return testResult;
  }

  async runTestSuite(tests) {
    this.results.startTime = Date.now();
    
    console.log(`Running ${tests.length} tests...`);
    
    for (const test of tests) {
      if (typeof test === 'function') {
        await this.runTest(test.name || 'Anonymous Test', test);
      } else if (typeof test === 'object' && test.name && test.fn) {
        await this.runTest(test.name, test.fn, test.options);
      }
    }
    
    this.results.endTime = Date.now();
    this.results.duration = this.results.endTime - this.results.startTime;
    
    return this.results;
  }

  async cleanup() {
    try {
      console.log('\nCleaning up test environment...');
      
      // Close browser
      if (this.browser) {
        await this.browser.close();
      }
      
      // Run teardown
      await this.teardown.teardown(this.results);
      
      // Stop mock store
      await this.setup.teardown();
      
      console.log('Cleanup complete');
    } catch (error) {
      console.error('Cleanup error:', error);
    }
  }

  async run(tests) {
    try {
      await this.initialize();
      const results = await this.runTestSuite(tests);
      
      // Print summary
      console.log('\n' + '='.repeat(50));
      console.log('TEST SUMMARY');
      console.log('='.repeat(50));
      console.log(`Total tests: ${results.tests.length}`);
      console.log(`Passed: ${results.passed}`);
      console.log(`Failed: ${results.failed}`);
      console.log(`Duration: ${(results.duration / 1000).toFixed(2)}s`);
      console.log('='.repeat(50));
      
      return results;
    } finally {
      await this.cleanup();
    }
  }

  // Helper method to run the actual scraper
  async runScraper(options = {}) {
    const { spawn } = require('child_process');
    
    return new Promise((resolve, reject) => {
      const env = {
        ...process.env,
        NODE_ENV: 'test',
        HEADLESS: 'true',
        AWS_ACCESS_KEY_ID: 'test-key',
        AWS_SECRET_ACCESS_KEY: 'test-secret',
        S3_BUCKET: 'test-bucket',
        EXAMPLE_SHOP_USERNAME: options.username || 'scraper_test',
        EXAMPLE_SHOP_PASSWORD: options.password || 'scraper_pass',
        LOCAL_MODE: 'true', // Don't actually upload to S3
        ...options.env
      };
      
      const scraperProcess = spawn('npm', ['start'], {
        cwd: this.config.scraperPath,
        env
      });
      
      let output = '';
      let errorOutput = '';
      
      scraperProcess.stdout.on('data', (data) => {
        output += data.toString();
        console.log(`Scraper: ${data}`);
      });
      
      scraperProcess.stderr.on('data', (data) => {
        errorOutput += data.toString();
        console.error(`Scraper Error: ${data}`);
      });
      
      scraperProcess.on('exit', (code) => {
        if (code === 0) {
          resolve({ success: true, output, errorOutput });
        } else {
          reject(new Error(`Scraper exited with code ${code}\n${errorOutput}`));
        }
      });
      
      scraperProcess.on('error', (error) => {
        reject(error);
      });
      
      // Set timeout
      setTimeout(() => {
        scraperProcess.kill('SIGTERM');
        reject(new Error('Scraper execution timeout'));
      }, options.timeout || 60000);
    });
  }
}

module.exports = TestRunner;