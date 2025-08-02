# E-commerce Scraper Integration Tests

This directory contains comprehensive integration tests for the e-commerce web scraper, including a mock React application that simulates the target website.

## Overview

The test suite consists of:
- **Mock Store**: A React application that mimics the Example Shop website
- **Integration Tests**: Full browser-based tests using Puppeteer
- **Screenshot Capture**: Automatic screenshots at each test step
- **Data Validation**: Comprehensive validation of scraped data

## Directory Structure

```
tests/
├── integration/          # Integration test infrastructure
│   ├── scenarios/       # Test scenarios
│   │   ├── fullWorkflow.test.js
│   │   ├── authentication.test.js
│   │   └── productExtraction.test.js
│   ├── utils/           # Test utilities
│   │   ├── screenshotHelper.js
│   │   ├── mockStoreManager.js
│   │   └── dataValidator.js
│   ├── setup.js         # Test environment setup
│   ├── teardown.js      # Test cleanup
│   └── testRunner.js    # Main test runner
├── fixtures/            # Test data
│   └── testCredentials.json
├── results/             # Test outputs
│   ├── screenshots/     # Test screenshots
│   ├── reports/         # Test reports
│   └── data/           # Extracted data
└── runTests.js         # Test entry point
```

## Mock Store

The mock store is a React application located in `./integration/mock-store/` that provides:

- **Authentication**: Login page with test credentials
- **Product Catalog**: 100 mock products with pagination
- **Proper Selectors**: All CSS selectors match the scraper's expectations
- **Responsive Design**: Works across different viewport sizes

### Mock Store Features

1. **Login Page** (`/login`)
   - Username field: `#username`
   - Password field: `#password`
   - Submit button: `#login-submit`

2. **Dashboard** (`/dashboard`)
   - Shows after successful login
   - Contains `.dashboard` element

3. **Products Page** (`/products`)
   - Product items: `.product-item`
   - Product ID: `[data-product-id]`
   - Product name: `.product-name`
   - Product price: `.product-price`
   - Availability: `.availability`
   - Pagination: `.pagination .next-page`

## Running Tests

### Prerequisites

1. Install dependencies:
```bash
npm run test:setup
```

2. Build the scraper:
```bash
npm run build
```

### Run All Tests

```bash
npm run test:integration
```

### Run Tests with Visible Browser

```bash
npm run test:integration:debug
```

### Run Individual Tests

```bash
# Start mock store manually
cd integration/mock-store && npm start

# Run specific test
node tests/runTests.js --test=authentication
```

## Test Scenarios

### 1. Full Workflow Test
Tests the complete scraper workflow:
- Homepage navigation
- Login process
- Dashboard verification
- Product page navigation
- Data extraction with pagination
- Data validation
- Output generation

### 2. Authentication Test
Tests various authentication scenarios:
- Valid credentials
- Invalid credentials
- Empty credentials
- Session management

### 3. Product Extraction Test
Tests data extraction capabilities:
- Selector verification
- Single product extraction
- Bulk extraction
- Data transformation
- Responsive behavior
- Edge cases

## Test Credentials

Valid test credentials (from `fixtures/testCredentials.json`):
- Username: `test_user` / Password: `test_pass`
- Username: `scraper_test` / Password: `scraper_pass`
- Username: `admin` / Password: `admin123`

## Screenshots

Screenshots are automatically captured at key points:
- Before/after each major action
- On test failures
- At pagination steps

Screenshots are saved to: `tests/results/screenshots/`

Format: `{test_name}_{step}_{timestamp}.png`

## Test Reports

After test execution, reports are generated:
- **JSON Report**: Detailed test results
- **HTML Report**: Visual test report with screenshots

Reports are saved to: `tests/results/reports/`

## Environment Variables

- `HEADLESS`: Set to `false` to see browser (default: `true`)
- `MOCK_STORE_PORT`: Port for mock store (default: `3000`)
- `SLOW_MO`: Slow down browser actions in ms (default: `0`)
- `DEVTOOLS`: Open browser DevTools (default: `false`)

## Troubleshooting

### Mock Store Won't Start
```bash
# Check if port is in use
lsof -i :3000

# Install dependencies
cd integration/mock-store && npm install
```

### Tests Timeout
- Increase timeout in `testRunner.js`
- Check if mock store is running
- Verify network connectivity

### Screenshots Not Saving
- Ensure `tests/results/screenshots/` directory exists
- Check file permissions

## Extending Tests

To add new test scenarios:

1. Create new test file in `tests/integration/scenarios/`
2. Export async function that accepts `(page, context)`
3. Use `ScreenshotHelper` for capturing screenshots
4. Return results object with `success` and `screenshots`
5. Add test to `runTests.js`

Example:
```javascript
async function myNewTest(page, context) {
  const { mockStoreUrl } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'my_new_test'
  });
  
  // Test implementation
  
  return {
    success: true,
    screenshots: screenshotHelper.getScreenshots()
  };
}
```

## CI/CD Integration

For CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run Integration Tests
  run: |
    npm run test:setup
    npm run build
    npm run test:integration
  env:
    HEADLESS: true
```

## Performance Considerations

- Tests run in parallel where possible
- Mock store starts once for all tests
- Browser instance is reused between tests
- Screenshots are compressed

## Future Enhancements

- [ ] Visual regression testing
- [ ] Performance benchmarking
- [ ] Network condition simulation
- [ ] Multi-browser testing
- [ ] Load testing with multiple scrapers
- [ ] API mocking for error scenarios