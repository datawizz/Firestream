# E-commerce Scraper Testing Guide

## Table of Contents

1. [Overview](#overview)
2. [Test Architecture](#test-architecture)
3. [Mock Store](#mock-store)
4. [Running Tests](#running-tests)
5. [Test Scenarios](#test-scenarios)
6. [Writing New Tests](#writing-new-tests)
7. [Test Utilities](#test-utilities)
8. [CI/CD Integration](#cicd-integration)
9. [Performance Testing](#performance-testing)
10. [Troubleshooting Tests](#troubleshooting-tests)

## Overview

The E-commerce Scraper includes a comprehensive testing framework featuring:

- **Mock Store**: Full React application simulating the target website
- **Integration Tests**: End-to-end workflow validation
- **Screenshot Capture**: Visual debugging and validation
- **Data Validation**: Automated data quality checks
- **Performance Benchmarking**: Execution time tracking

## Test Architecture

```
tests/
├── integration/              # Integration test suite
│   ├── scenarios/           # Test scenarios
│   │   ├── authentication.test.js
│   │   ├── productExtraction.test.js
│   │   └── fullWorkflow.test.js
│   ├── utils/               # Test utilities
│   │   ├── screenshotHelper.js
│   │   ├── mockStoreManager.js
│   │   ├── dataValidator.js
│   │   └── navigationHelper.js
│   ├── setup.js             # Test environment setup
│   ├── teardown.js          # Test cleanup
│   └── testRunner.js        # Main test runner
├── fixtures/                # Test data
│   └── testCredentials.json
├── results/                 # Test outputs
│   ├── screenshots/         # Test screenshots
│   ├── reports/            # Test reports
│   ├── data/               # Extracted data
│   └── archives/           # Historical results
└── runTests.js             # Test entry point
```

## Mock Store

### Overview

The mock store is a React application that simulates the Example Shop e-commerce site.

### Features

- **Authentication System**: Login with multiple test accounts
- **Product Catalog**: 100+ mock products
- **Pagination**: 20 products per page
- **Responsive Design**: Mobile and desktop layouts
- **Consistent Selectors**: Matches production selectors

### Running Mock Store

```bash
# Install dependencies
cd tests/integration/mock-store
npm install

# Start development server
npm start

# Build for production
npm run build
```

### Mock Store Routes

| Route | Description | Key Elements |
|-------|-------------|--------------|
| `/` | Homepage | `.home`, `.login-button` |
| `/login` | Login page | `#username`, `#password`, `#login-submit` |
| `/dashboard` | User dashboard | `.dashboard`, `.user-menu` |
| `/products` | Product catalog | `.product-item`, `.pagination` |

### Test Credentials

```json
{
  "valid": [
    { "username": "test_user", "password": "test_pass" },
    { "username": "scraper_test", "password": "scraper_pass" },
    { "username": "admin", "password": "admin123" }
  ],
  "invalid": [
    { "username": "wrong_user", "password": "wrong_pass" },
    { "username": "", "password": "" }
  ]
}
```

## Running Tests

### Quick Start

```bash
# Run all integration tests
npm run test:integration

# Run with visible browser (debugging)
npm run test:integration:debug

# Run specific test scenario
node tests/runTests.js --test=authentication

# Run with custom options
HEADLESS=false SLOW_MO=100 npm run test:integration
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HEADLESS` | Run browser in headless mode | `true` |
| `MOCK_STORE_PORT` | Mock store port | `3000` |
| `SLOW_MO` | Slow down browser actions (ms) | `0` |
| `DEVTOOLS` | Open browser DevTools | `false` |
| `SCREENSHOT_ON_FAILURE` | Capture screenshots on failure | `true` |

### Test Output

```
Test Results Location:
- Reports: tests/results/reports/test-report-[timestamp].html
- Screenshots: tests/results/screenshots/
- Data: tests/results/data/
- Archives: tests/results/archives/[date]/
```

## Test Scenarios

### 1. Authentication Test

Tests various authentication scenarios.

**Test Cases:**

```javascript
// Valid login
- Navigate to homepage
- Click login button
- Enter valid credentials
- Submit form
- Verify dashboard access

// Invalid login
- Navigate to login page
- Enter invalid credentials
- Submit form
- Verify error message
- Confirm no dashboard access

// Empty credentials
- Navigate to login page
- Submit without entering credentials
- Verify validation errors

// Session persistence
- Login successfully
- Navigate away
- Return to protected page
- Verify still authenticated
```

### 2. Product Extraction Test

Validates data extraction capabilities.

**Test Cases:**

```javascript
// Single page extraction
- Navigate to products page
- Extract product data
- Validate data structure
- Verify all fields present

// Multi-page extraction
- Navigate to products page
- Extract first page data
- Click next page
- Extract second page data
- Verify pagination works

// Edge cases
- Empty product list
- Missing product fields
- Special characters in data
- Very long product names
```

### 3. Full Workflow Test

End-to-end workflow validation.

**Test Flow:**

```javascript
1. Homepage Navigation
   - Load homepage
   - Verify page loaded
   - Screenshot: homepage

2. Authentication
   - Click login
   - Enter credentials
   - Submit form
   - Verify dashboard
   - Screenshot: dashboard

3. Product Navigation
   - Navigate to products
   - Wait for products to load
   - Screenshot: products page

4. Data Extraction
   - Extract all products
   - Handle pagination
   - Screenshot: each page

5. Data Validation
   - Validate schema
   - Check required fields
   - Verify data types

6. Data Export
   - Serialize to JSON
   - Save to results directory
   - Verify file created
```

## Writing New Tests

### Test Structure

```javascript
// tests/integration/scenarios/myTest.test.js

const { ScreenshotHelper } = require('../utils/screenshotHelper');
const { NavigationHelper } = require('../utils/navigationHelper');
const { DataValidator } = require('../utils/dataValidator');

async function myTestScenario(page, context) {
  const { mockStoreUrl, credentials } = context;
  
  // Initialize helpers
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'My Test Scenario'
  });
  
  const navigationHelper = new NavigationHelper(page);
  const validator = new DataValidator();
  
  try {
    // Test implementation
    await navigationHelper.goto(mockStoreUrl);
    await screenshotHelper.capture('homepage');
    
    // Your test logic here
    
    return {
      success: true,
      message: 'Test completed successfully',
      screenshots: screenshotHelper.getScreenshots(),
      data: extractedData
    };
    
  } catch (error) {
    await screenshotHelper.capture('error');
    
    return {
      success: false,
      message: error.message,
      screenshots: screenshotHelper.getScreenshots(),
      error
    };
  }
}

module.exports = { myTestScenario };
```

### Adding to Test Suite

```javascript
// tests/runTests.js

const { myTestScenario } = require('./integration/scenarios/myTest.test');

const testScenarios = {
  authentication: authenticationTest,
  productExtraction: productExtractionTest,
  fullWorkflow: fullWorkflowTest,
  myTest: myTestScenario  // Add your test here
};
```

### Best Practices

1. **Use Helpers**: Leverage existing test utilities
2. **Capture Screenshots**: At key points and on failure
3. **Validate Data**: Check structure and content
4. **Handle Errors**: Graceful failure with debugging info
5. **Clean Up**: Reset state between tests

## Test Utilities

### ScreenshotHelper

Manages screenshot capture and organization.

```javascript
const screenshotHelper = new ScreenshotHelper(page, {
  testName: 'My Test',
  outputDir: './screenshots'
});

// Capture screenshot
await screenshotHelper.capture('step-name');

// Capture with options
await screenshotHelper.capture('full-page', {
  fullPage: true,
  clip: { x: 0, y: 0, width: 800, height: 600 }
});

// Get all screenshots
const screenshots = screenshotHelper.getScreenshots();
```

### NavigationHelper

Simplifies page navigation with retry logic.

```javascript
const navHelper = new NavigationHelper(page);

// Navigate with retry
await navHelper.goto('https://example.com', {
  waitUntil: 'networkidle2',
  timeout: 30000,
  retries: 3
});

// Wait for navigation after action
await navHelper.clickAndNavigate('.submit-button');

// Check if element exists
const exists = await navHelper.elementExists('.product');
```

### DataValidator

Validates extracted data against schemas.

```javascript
const validator = new DataValidator();

// Define schema
const productSchema = {
  id: { type: 'string', required: true },
  name: { type: 'string', required: true },
  price: { type: 'number', required: true, min: 0 },
  inStock: { type: 'boolean', required: false }
};

// Validate single item
const result = validator.validateItem(product, productSchema);

// Validate array
const results = validator.validateArray(products, productSchema);

// Get validation report
const report = validator.getValidationReport();
```

### MockStoreManager

Manages mock store lifecycle.

```javascript
const mockStore = new MockStoreManager({
  port: 3000,
  directory: './tests/integration/mock-store'
});

// Start mock store
await mockStore.start();

// Check if running
const isRunning = await mockStore.isRunning();

// Stop mock store
await mockStore.stop();

// Reset data
await mockStore.reset();
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Integration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        
    - name: Install dependencies
      run: |
        npm ci
        npm run test:setup
        
    - name: Build scraper
      run: npm run build
      
    - name: Run integration tests
      run: npm run test:integration
      env:
        HEADLESS: true
        
    - name: Upload test results
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: |
          tests/results/reports/
          tests/results/screenshots/
          
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage/lcov.info
```

### Jenkins Pipeline

```groovy
pipeline {
  agent any
  
  stages {
    stage('Setup') {
      steps {
        sh 'npm ci'
        sh 'npm run test:setup'
      }
    }
    
    stage('Build') {
      steps {
        sh 'npm run build'
      }
    }
    
    stage('Test') {
      steps {
        sh 'npm run test:integration'
      }
    }
    
    stage('Archive Results') {
      steps {
        archiveArtifacts artifacts: 'tests/results/**/*'
        publishHTML([
          reportDir: 'tests/results/reports',
          reportFiles: '*.html',
          reportName: 'Test Report'
        ])
      }
    }
  }
  
  post {
    always {
      cleanWs()
    }
  }
}
```

### Docker Testing

```dockerfile
# Dockerfile.test
FROM node:18-alpine

# Install Chromium
RUN apk add --no-cache chromium

ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
ENV PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium-browser

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

CMD ["npm", "run", "test:integration"]
```

## Performance Testing

### Running Performance Tests

```bash
# Run performance benchmarks
npm run test:performance

# Run with specific iterations
ITERATIONS=10 npm run test:performance

# Generate performance report
npm run test:performance:report
```

### Performance Metrics

```javascript
// tests/performance-test.js

const metrics = {
  // Execution time
  totalDuration: endTime - startTime,
  effectDurations: effectTimings,
  
  // Resource usage
  memoryUsage: process.memoryUsage(),
  cpuUsage: process.cpuUsage(),
  
  // Data metrics
  itemsExtracted: products.length,
  pagesProcessed: pageCount,
  errorsEncountered: errors.length,
  
  // Rate metrics
  itemsPerSecond: products.length / (duration / 1000),
  pagesPerMinute: pageCount / (duration / 60000)
};
```

### Benchmarking

```javascript
// Benchmark configuration
const benchmarkConfig = {
  warmupRuns: 2,
  testRuns: 5,
  scenarios: [
    { name: 'Single Page', pages: 1 },
    { name: 'Five Pages', pages: 5 },
    { name: 'All Pages', pages: -1 }
  ]
};

// Run benchmarks
const results = await runBenchmarks(benchmarkConfig);

// Generate report
generateBenchmarkReport(results);
```

## Troubleshooting Tests

### Common Issues

#### Mock Store Won't Start

```bash
# Check port availability
lsof -i :3000

# Kill existing process
kill -9 $(lsof -t -i:3000)

# Use different port
MOCK_STORE_PORT=3001 npm run test:integration
```

#### Tests Timeout

```javascript
// Increase timeout in test
page.setDefaultTimeout(60000); // 60 seconds

// Increase Jest timeout
jest.setTimeout(120000); // 2 minutes

// Add explicit waits
await page.waitForSelector('.element', { timeout: 30000 });
```

#### Screenshot Issues

```bash
# Create screenshots directory
mkdir -p tests/results/screenshots

# Check permissions
chmod -R 755 tests/results

# Debug screenshot path
console.log('Screenshot path:', screenshotPath);
```

#### Flaky Tests

```javascript
// Add retry logic
for (let attempt = 0; attempt < 3; attempt++) {
  try {
    await page.click('.button');
    break;
  } catch (error) {
    if (attempt === 2) throw error;
    await page.waitForTimeout(1000);
  }
}

// Wait for stable state
await page.waitForLoadState('networkidle');
await page.waitForTimeout(500);
```

### Debug Mode

```bash
# Run with full debugging
DEBUG=* HEADLESS=false DEVTOOLS=true SLOW_MO=250 \
  node --inspect tests/runTests.js

# Connect Chrome DevTools to Node.js debugger
# Open chrome://inspect in Chrome
```

### Test Isolation

```javascript
// Ensure clean state
beforeEach(async () => {
  // Clear cookies
  const cookies = await page.cookies();
  await page.deleteCookie(...cookies);
  
  // Clear local storage
  await page.evaluate(() => localStorage.clear());
  
  // Reset mock store
  await mockStore.reset();
});
```

### Logging

```javascript
// Enable verbose logging
const logger = createLogger({
  level: 'debug',
  format: format.combine(
    format.timestamp(),
    format.json()
  ),
  transports: [
    new transports.File({ filename: 'test.log' }),
    new transports.Console()
  ]
});

// Log test steps
logger.info('Starting test', { test: 'authentication' });
logger.debug('Clicking element', { selector: '.button' });
logger.error('Test failed', { error: error.message });
```