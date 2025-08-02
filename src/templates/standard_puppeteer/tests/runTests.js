#!/usr/bin/env node

const path = require('path');
const dotenv = require('dotenv');

// Load test environment variables
dotenv.config({ path: path.join(__dirname, '.env.test') });

const TestRunner = require('./integration/testRunner');
const fullWorkflowTest = require('./integration/scenarios/fullWorkflow.test');
const authenticationTest = require('./integration/scenarios/authentication.test');
const productExtractionTest = require('./integration/scenarios/productExtraction.test');
const cookieApiExtractionTest = require('./integration/scenarios/cookieApiExtraction.test');
const fullCookieWorkflowTest = require('./integration/scenarios/fullCookieWorkflow.test');
const cookieApiWorkflowTest = require('./integration/scenarios/cookieApiWorkflow.test');

async function main() {
  console.log('E-commerce Scraper Integration Tests');
  console.log('====================================\n');

  const testRunner = new TestRunner({
    headless: process.env.HEADLESS !== 'false',
    mockStorePort: process.env.MOCK_STORE_PORT || 3000,
    apiPort: process.env.API_PORT || 3001
  });

  // Define test suite
  const tests = [
    {
      name: 'Authentication Tests',
      fn: authenticationTest,
      options: {}
    },
    {
      name: 'Product Extraction Tests',
      fn: productExtractionTest,
      options: {}
    },
    {
      name: 'Full Workflow Test (DOM-based)',
      fn: fullWorkflowTest,
      options: {}
    },
    {
      name: 'Cookie API Extraction Tests',
      fn: cookieApiExtractionTest,
      options: {}
    },
    {
      name: 'Full Cookie-to-API Workflow Test',
      fn: fullCookieWorkflowTest,
      options: {}
    },
    {
      name: 'Cookie-API Workflow (Effects Framework)',
      fn: cookieApiWorkflowTest,
      options: {}
    }
  ];

  try {
    const results = await testRunner.run(tests);
    
    // Exit with appropriate code
    process.exit(results.failed > 0 ? 1 : 0);
  } catch (error) {
    console.error('Test execution failed:', error);
    process.exit(1);
  }
}

// Run tests if this is the main module
if (require.main === module) {
  main().catch(error => {
    console.error('Unhandled error:', error);
    process.exit(1);
  });
}

module.exports = { main };