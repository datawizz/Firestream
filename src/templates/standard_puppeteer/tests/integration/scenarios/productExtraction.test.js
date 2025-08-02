const ScreenshotHelper = require('../utils/screenshotHelper');
const NavigationHelper = require('../utils/navigationHelper');
const DataValidator = require('../utils/dataValidator');
const testCredentials = require('../../fixtures/testCredentials.json');

async function productExtractionTest(page, context) {
  const { mockStoreUrl, mockStoreManager } = context;
  const screenshotHelper = new ScreenshotHelper(page, {
    testName: 'product_extraction',
    timestamp: Date.now()
  });
  const navigationHelper = new NavigationHelper(page, mockStoreManager);
  const dataValidator = new DataValidator();
  
  const results = {
    screenshots: []
  };

  try {
    // First navigate to the home page
    console.log('Navigating to mock store home page...');
    await page.goto(mockStoreUrl, { waitUntil: 'networkidle2' });
    
    // Re-inject test helpers after navigation
    await mockStoreManager.injectTestHelpers(page);
    
    // Use navigationHelper to login and navigate
    await navigationHelper.loginAndNavigate(testCredentials.valid, '/dashboard');
    
    // Ensure dashboard is fully loaded with content
    await navigationHelper.waitForContentToLoad('.dashboard', {
      timeout: 10000,
      minLength: 20
    });
    
    // Wait for dashboard to be stable
    await page.waitForTimeout(2000);
    
    // Navigate to products using navigationHelper
    console.log('Navigating to products page...');
    
    // Debug: Check navigation links
    const navLinks = await page.evaluate(() => {
      const links = Array.from(document.querySelectorAll('nav a, .dashboard a, button'));
      return links.map(link => ({
        href: link.href || link.getAttribute('href') || 'no-href',
        text: link.textContent?.trim(),
        tagName: link.tagName,
        className: link.className,
        visible: link.offsetParent !== null
      }));
    });
    console.log('Available navigation elements:', JSON.stringify(navLinks, null, 2));
    
    // Check current URL
    const currentUrl = page.url();
    console.log('Current URL before products navigation:', currentUrl);
    
    // Try multiple strategies to navigate to products
    let navigatedToProducts = false;
    
    // Strategy 1: Try clicking the "View All Products" button on dashboard
    try {
      console.log('Looking for "View All Products" button...');
      
      // Wait for button containing text "View All Products"
      await page.waitForFunction(
        () => {
          const buttons = Array.from(document.querySelectorAll('button'));
          return buttons.some(btn => btn.textContent && btn.textContent.includes('View All Products'));
        },
        { timeout: 5000 }
      );
      
      // Use page.evaluate to click the button that navigates to products
      await page.evaluate(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        const productsButton = buttons.find(btn => btn.textContent && btn.textContent.includes('View All Products'));
        if (productsButton) {
          productsButton.click();
        } else {
          throw new Error('View All Products button not found');
        }
      });
      
      // Wait for navigation to complete
      await page.waitForFunction(
        () => window.location.href.includes('/products'),
        { timeout: 10000 }
      );
      
      // Wait for products page to load
      await page.waitForSelector('.products-page', { timeout: 10000 });
      await navigationHelper.waitForContentToLoad('.product-item', {
        timeout: 10000,
        minLength: 10
      });
      
      navigatedToProducts = true;
      console.log('Successfully navigated to products page via button');
    } catch (e) {
      console.log('Strategy 1 (button click) failed:', e.message);
    }
    
    // Strategy 2: Try clicking products link if it exists
    if (!navigatedToProducts) {
      try {
        const productsLink = await page.$('a[href="/products"]');
        if (productsLink) {
          console.log('Found products link, clicking...');
          await navigationHelper.clickAndWaitForNavigation('a[href="/products"]', {
            waitForUrl: '/products',
            waitForSelector: '.product-item',
            timeout: 10000
          });
          navigatedToProducts = true;
        }
      } catch (e) {
        console.log('Strategy 2 (link click) failed:', e.message);
      }
    }
    
    // Strategy 3: If no link/button found or click failed, navigate directly
    if (!navigatedToProducts) {
      console.log('Button/link not found or click failed, navigating directly...');
      const baseUrl = currentUrl.split('/').slice(0, 3).join('/');
      await navigationHelper.navigateToUrl(`${baseUrl}/products`, {
        waitForSelector: '.product-item',
        timeout: 10000
      });
      navigatedToProducts = true;
    }
    
    // Wait for products content to be fully loaded
    await navigationHelper.waitForContentToLoad('.products-page', {
      timeout: 5000,
      minLength: 50
    });
    
    // Debug: Check what's on the products page
    const pageState = await page.evaluate(() => ({
      url: window.location.href,
      hasProductItems: !!document.querySelector('.product-item'),
      productCount: document.querySelectorAll('.product-item').length,
      pageClasses: document.body.className,
      mainContent: document.querySelector('.main-content')?.innerHTML.substring(0, 200)
    }));
    console.log('Products page state:', JSON.stringify(pageState, null, 2));
    
    // Ensure products are loaded with content
    await navigationHelper.waitForReactComponent('.product-item', {
      timeout: 10000,
      checkContent: true
    });
    
    // Wait for all product items to have content
    await page.waitForFunction(
      () => {
        const items = document.querySelectorAll('.product-item');
        if (items.length === 0) return false;
        
        // Check that all items have essential content
        for (const item of items) {
          const name = item.querySelector('.product-name')?.textContent || '';
          const price = item.querySelector('.product-price')?.textContent || '';
          if (name.trim().length === 0 || price.trim().length === 0) {
            return false;
          }
        }
        return true;
      },
      { timeout: 5000 }
    );

    // Test 1: Verify selectors exist
    console.log('Test 1: Verifying all required selectors...');
    const selectors = {
      productItem: '.product-item',
      productId: '[data-product-id]',
      productName: '.product-name',
      productPrice: '.product-price',
      availability: '.availability'
    };

    for (const [name, selector] of Object.entries(selectors)) {
      const element = await page.$(selector);
      if (!element) {
        throw new Error(`Required selector not found: ${name} (${selector})`);
      }
    }

    results.screenshots.push(await screenshotHelper.capture('selectors_verified'));

    // Test 2: Extract single product details
    console.log('Test 2: Extracting single product details...');
    const firstProduct = await page.evaluate(() => {
      const item = document.querySelector('.product-item');
      if (!item) return null;

      return {
        id: item.getAttribute('data-product-id'),
        name: item.querySelector('.product-name')?.textContent,
        price: item.querySelector('.product-price')?.textContent,
        availability: item.querySelector('.availability')?.textContent,
        html: item.innerHTML
      };
    });

    console.log('First product extracted:', firstProduct);

    // Test 3: Extract all products on page
    console.log('Test 3: Extracting all products on current page...');
    const allProducts = await page.evaluate(() => {
      const products = [];
      document.querySelectorAll('.product-item').forEach(item => {
        products.push({
          id: item.getAttribute('data-product-id'),
          name: item.querySelector('.product-name')?.textContent || '',
          price: item.querySelector('.product-price')?.textContent || '',
          availability: item.querySelector('.availability')?.textContent || ''
        });
      });
      return products;
    });

    console.log(`Extracted ${allProducts.length} products`);
    results.screenshots.push(await screenshotHelper.capture('all_products_extracted'));

    // Test 4: Verify data transformation
    console.log('Test 4: Testing data transformation...');
    const transformedProducts = allProducts.map(product => ({
      id: product.id,
      name: product.name.trim(),
      price: parseFloat(product.price.replace('$', '')),
      inStock: product.availability.toLowerCase().includes('in stock')
    }));

    // Validate transformed data
    const validation = dataValidator.validateProductList(transformedProducts);
    if (validation.invalid > 0) {
      console.warn('Validation issues found:', validation);
    }

    // Test 5: Test extraction with different viewport sizes
    console.log('Test 5: Testing responsive extraction...');
    const viewports = [
      { width: 1920, height: 1080, name: 'desktop' },
      { width: 768, height: 1024, name: 'tablet' },
      { width: 375, height: 667, name: 'mobile' }
    ];

    for (const viewport of viewports) {
      await page.setViewport({ width: viewport.width, height: viewport.height });
      
      // Wait for layout to adjust and content to reflow
      await navigationHelper.waitForContentToLoad('.products-page', {
        timeout: 3000,
        minLength: 50
      });
      
      // Verify products are still visible and have content
      await page.waitForFunction(
        () => {
          const items = document.querySelectorAll('.product-item');
          if (items.length === 0) return false;
          
          // Check first item has content
          const firstItem = items[0];
          const name = firstItem.querySelector('.product-name')?.textContent || '';
          return name.trim().length > 0;
        },
        { timeout: 3000 }
      );
      
      const responsiveProducts = await page.evaluate(() => {
        return document.querySelectorAll('.product-item').length;
      });
      
      console.log(`  - ${viewport.name}: ${responsiveProducts} products visible`);
      results.screenshots.push(
        await screenshotHelper.capture(`extraction_${viewport.name}`)
      );
    }

    // Test 6: Edge cases
    console.log('Test 6: Testing edge cases...');
    
    // Test with product that might have missing fields
    const edgeCaseTests = await page.evaluate(() => {
      const tests = [];
      
      // Check for any products with empty fields
      document.querySelectorAll('.product-item').forEach((item, index) => {
        const id = item.getAttribute('data-product-id');
        const name = item.querySelector('.product-name')?.textContent;
        const price = item.querySelector('.product-price')?.textContent;
        
        if (!id || !name || !price) {
          tests.push({
            index,
            issue: 'missing_field',
            details: { id: !!id, name: !!name, price: !!price }
          });
        }
      });
      
      return tests;
    });

    if (edgeCaseTests.length > 0) {
      console.warn('Edge cases found:', edgeCaseTests);
    }

    return {
      success: true,
      screenshots: screenshotHelper.getScreenshots().map(s => s.filepath),
      extractedCount: allProducts.length,
      validationResult: validation,
      edgeCases: edgeCaseTests
    };

  } catch (error) {
    await screenshotHelper.captureOnError(error, 'extraction_error');
    throw error;
  }
}

productExtractionTest.description = 'Tests product data extraction capabilities and selector reliability';

module.exports = productExtractionTest;