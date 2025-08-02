const path = require('path');
const fs = require('fs');
const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');
const DataValidator = require('../utils/dataValidator');
const testCredentials = require('../../fixtures/testCredentials.json');

async function fullWorkflowTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'full_workflow',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);
  const dataValidator = new DataValidator();
  
  const results = {
    screenshots: [],
    data: null,
    validation: null
  };

  try {
    // Step 1: Navigate to homepage
    console.log('Step 1: Navigating to homepage...');
    await page.goto(mockStoreUrl, { waitUntil: 'networkidle2' });
    
    // Wait for React app to mount
    await page.waitForFunction(
      () => {
        const root = document.querySelector('#root');
        return root && root.children.length > 0;
      },
      { timeout: 10000 }
    );
    await page.waitForTimeout(1000); // Additional wait for complete render
    
    results.screenshots.push(await screenshotHelper.capture('01_homepage'));

    // Verify homepage loaded
    const title = await page.title();
    if (!title.includes('Example Shop')) {
      throw new Error('Homepage did not load correctly');
    }

    // Step 2: Click login button
    console.log('Step 2: Clicking login button...');
    await page.waitForSelector('.login-button', { timeout: 5000 });
    results.screenshots.push(await screenshotHelper.capture('02_before_login_click'));
    
    await navigationHelper.clickAndWaitForNavigation('.login-button', {
      waitForUrl: '/login',
      waitForSelector: '#username'
    });
    
    results.screenshots.push(await screenshotHelper.capture('03_login_page'));

    // Step 3: Enter credentials
    console.log('Step 3: Entering credentials...');
    await page.waitForSelector('#username', { timeout: 5000 });
    
    // Enter credentials with proper React input events
    await page.focus('#username');
    await page.evaluate((val) => {
      const input = document.querySelector('#username');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, testCredentials.scraper.username);
    
    await page.focus('#password');
    await page.evaluate((val) => {
      const input = document.querySelector('#password');
      const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      nativeInputValueSetter.call(input, val);
      const event = new Event('input', { bubbles: true });
      input.dispatchEvent(event);
    }, testCredentials.scraper.password);
    results.screenshots.push(await screenshotHelper.capture('04_credentials_entered'));

    // Step 4: Submit login
    console.log('Step 4: Submitting login...');
    await page.click('#login-submit');
    
    // Step 5: Wait for dashboard navigation
    console.log('Step 5: Waiting for dashboard...');
    await page.waitForTimeout(1500); // Wait for login processing
    await page.waitForFunction(
      () => window.location.href.includes('/dashboard'),
      { timeout: 10000 }
    );
    await page.waitForSelector('.dashboard', { timeout: 10000 });
    results.screenshots.push(await screenshotHelper.capture('05_dashboard'));

    // Verify login success
    const isLoggedIn = await page.evaluate(() => !!sessionStorage.getItem('user'));
    if (!isLoggedIn) {
      throw new Error('Login was not successful');
    }

    // Step 6: Navigate to products
    console.log('Step 6: Navigating to products page...');
    
    // Click the products link using evaluate to avoid visibility issues
    await page.evaluate(() => {
      const link = document.querySelector('a[href="/products"]');
      if (link) link.click();
    });
    
    // Wait for navigation to complete
    await page.waitForFunction(
      () => window.location.href.includes('/products'),
      { timeout: 5000 }
    );
    
    // Wait a bit for products to load
    await page.waitForTimeout(1000);
    
    await page.waitForSelector('.product-item', { timeout: 10000 });
    results.screenshots.push(await screenshotHelper.capture('06_products_page'));

    // Step 7: Extract products with pagination
    console.log('Step 7: Extracting products from all pages...');
    const allProducts = [];
    let currentPage = 1;
    const maxPages = 5;

    while (currentPage <= maxPages) {
      // Wait for products to load
      await page.waitForSelector('.product-item', { timeout: 5000 });
      
      // Extract products from current page
      const pageProducts = await page.evaluate(() => {
        const products = [];
        const items = document.querySelectorAll('.product-item');
        
        items.forEach(item => {
          const id = item.getAttribute('data-product-id');
          const name = item.querySelector('.product-name')?.textContent || '';
          const priceText = item.querySelector('.product-price')?.textContent || '0';
          const price = parseFloat(priceText.replace('$', ''));
          const availabilityText = item.querySelector('.availability')?.textContent || '';
          const inStock = availabilityText.toLowerCase().includes('in stock');
          
          products.push({ id, name, price, inStock });
        });
        
        return products;
      });

      console.log(`  - Page ${currentPage}: Found ${pageProducts.length} products`);
      allProducts.push(...pageProducts);
      results.screenshots.push(await screenshotHelper.capture(`07_products_page_${currentPage}`));

      // Check if there's a next page
      const hasNextPage = await page.evaluate(() => {
        const nextButton = document.querySelector('.pagination .next-page');
        return nextButton && !nextButton.disabled;
      });

      if (!hasNextPage || currentPage >= maxPages) {
        break;
      }

      // Click next page
      await page.click('.pagination .next-page');
      await page.waitForTimeout(1000); // Wait for page transition
      currentPage++;
    }

    console.log(`Total products extracted: ${allProducts.length}`);
    results.data = allProducts;

    // Step 8: Validate extracted data
    console.log('Step 8: Validating extracted data...');
    const validation = dataValidator.validateProductList(allProducts);
    results.validation = validation;

    if (validation.invalid > 0) {
      console.warn(`Data validation: ${validation.invalid} invalid products found`);
      validation.invalidProducts.forEach(({ index, errors }) => {
        console.warn(`  - Product ${index}: ${errors.join(', ')}`);
      });
    }

    // Step 9: Verify data completeness
    console.log('Step 9: Verifying data completeness...');
    if (allProducts.length === 0) {
      throw new Error('No products were extracted');
    }

    if (allProducts.length < 80) {
      console.warn(`Expected ~100 products but only found ${allProducts.length}`);
    }

    // Step 10: Save extracted data (simulate S3 upload)
    console.log('Step 10: Saving extracted data...');
    const outputDir = path.join(__dirname, '../../results/data');
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    const outputPath = path.join(outputDir, `products_${Date.now()}.json`);
    const outputData = {
      metadata: {
        workflow_id: 'ecommerce_product_scraper',
        execution_id: `test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        items_count: allProducts.length
      },
      products: allProducts
    };

    fs.writeFileSync(outputPath, JSON.stringify(outputData, null, 2));
    console.log(`Data saved to: ${outputPath}`);
    results.screenshots.push(await screenshotHelper.capture('10_final_state'));

    // Generate validation report
    const validationReport = dataValidator.generateValidationReport();

    return {
      success: true,
      screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
      productsExtracted: allProducts.length,
      validationReport,
      dataPath: outputPath
    };

  } catch (error) {
    await screenshotHelper.captureOnError(error, 'workflow_error');
    throw error;
  }
}

fullWorkflowTest.description = 'Tests the complete scraper workflow from login to data extraction';

module.exports = fullWorkflowTest;