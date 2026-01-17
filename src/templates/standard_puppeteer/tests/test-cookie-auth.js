#!/usr/bin/env node

const puppeteer = require('puppeteer');

async function testCookieAuth() {
  console.log('Testing Cookie Authentication...\n');
  
  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  
  const page = await browser.newPage();
  
  try {
    // Step 1: Login to frontend
    console.log('1. Logging into frontend...');
    await page.goto('http://localhost:3000/login', { waitUntil: 'networkidle2' });
    
    await page.type('#username', 'testuser');
    await page.type('#password', 'password123');
    
    await page.evaluate(() => {
      document.querySelector('form').submit();
    });
    
    await page.waitForFunction(() => window.location.href.includes('/dashboard'), { timeout: 5000 });
    console.log('   ✓ Login successful');
    
    // Step 2: Get cookies
    const cookies = await page.cookies();
    const authCookies = cookies.filter(c => 
      ['sessionId', 'authToken'].includes(c.name)
    );
    console.log(`   ✓ Found ${authCookies.length} auth cookies`);
    
    // Step 3: Navigate to API and set cookies
    console.log('\n2. Setting cookies for API domain...');
    await page.goto('http://localhost:3001/api/health', { waitUntil: 'domcontentloaded' });
    
    for (const cookie of authCookies) {
      await page.setCookie({
        ...cookie,
        domain: 'localhost',
        url: undefined // Remove URL to avoid conflicts
      });
    }
    console.log('   ✓ Cookies set for API domain');
    
    // Step 4: Test API calls
    console.log('\n3. Testing API calls...');
    
    const apiTests = [
      { name: 'Health Check', url: 'http://localhost:3001/api/health' },
      { name: 'Auth Status', url: 'http://localhost:3001/api/auth/status' },
      { name: 'Categories', url: 'http://localhost:3001/api/categories' },
      { name: 'Products', url: 'http://localhost:3001/api/products?page=1&limit=5' }
    ];
    
    for (const test of apiTests) {
      const response = await page.evaluate(async (url) => {
        const res = await fetch(url, {
          method: 'GET',
          credentials: 'include'
        });
        return {
          status: res.status,
          data: await res.json()
        };
      }, test.url);
      
      console.log(`   ${test.name}: ${response.status} - ${
        response.status === 200 ? '✓ Success' : '✗ Failed'
      }`);
      
      if (response.status !== 200) {
        console.log(`     Error: ${JSON.stringify(response.data)}`);
      }
    }
    
    // Step 5: Verify cookies work across navigations
    console.log('\n4. Testing cookie persistence...');
    await page.goto('http://localhost:3000/products', { waitUntil: 'networkidle2' });
    
    const finalCookies = await page.cookies();
    console.log(`   ✓ Cookies still present: ${finalCookies.length}`);
    
  } catch (error) {
    console.error('Test failed:', error);
  } finally {
    await browser.close();
  }
}

testCookieAuth();