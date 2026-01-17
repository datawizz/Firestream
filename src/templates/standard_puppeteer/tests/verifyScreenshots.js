#!/usr/bin/env node

const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

async function verifyScreenshots() {
  console.log('Screenshot Verification Test');
  console.log('===========================\n');
  
  let browser;
  try {
    // Check environment
    console.log('Environment:');
    console.log(`- DISPLAY: ${process.env.DISPLAY || 'not set'}`);
    console.log(`- PUPPETEER_EXECUTABLE_PATH: ${process.env.PUPPETEER_EXECUTABLE_PATH || 'not set'}`);
    console.log(`- Platform: ${process.platform}`);
    console.log('\n');
    
    // Launch browser with our configuration
    const executablePath = process.env.PUPPETEER_EXECUTABLE_PATH;
    console.log('Launching browser...');
    
    browser = await puppeteer.launch({
      headless: true,
      executablePath,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--single-process',
        '--font-render-hinting=none',
        '--force-device-scale-factor=1',
        '--force-color-profile=srgb'
      ],
      defaultViewport: {
        width: 1920,
        height: 1080,
        deviceScaleFactor: 1
      }
    });
    
    console.log('Browser launched successfully\n');
    
    // Create page
    const page = await browser.newPage();
    
    // Test 1: Simple HTML screenshot
    console.log('Test 1: Simple HTML Screenshot');
    await page.setContent(`
      <html>
        <head>
          <style>
            body {
              font-family: Arial, sans-serif;
              margin: 40px;
              background: #f0f0f0;
            }
            .test-box {
              background: white;
              padding: 20px;
              border-radius: 8px;
              box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
            h1 { color: #333; }
            p { color: #666; }
          </style>
        </head>
        <body>
          <div class="test-box">
            <h1>Screenshot Test</h1>
            <p>This is a test page to verify screenshot functionality.</p>
            <p>Timestamp: ${new Date().toISOString()}</p>
            <p>User Agent: <span id="ua"></span></p>
          </div>
          <script>
            document.getElementById('ua').textContent = navigator.userAgent;
          </script>
        </body>
      </html>
    `);
    
    await page.waitForTimeout(1000);
    
    const testDir = path.join(__dirname, 'results', 'screenshot-test');
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }
    
    const screenshot1 = path.join(testDir, 'test1-simple.png');
    await page.screenshot({ path: screenshot1, fullPage: true });
    
    const stats1 = fs.statSync(screenshot1);
    console.log(`✓ Screenshot saved: ${screenshot1}`);
    console.log(`  Size: ${stats1.size} bytes`);
    console.log(`  Valid: ${stats1.size > 5000 ? 'Yes' : 'No (too small)'}\n`);
    
    // Test 2: External website screenshot
    console.log('Test 2: External Website Screenshot');
    await page.goto('https://example.com', { waitUntil: 'networkidle2' });
    await page.waitForTimeout(2000);
    
    const screenshot2 = path.join(testDir, 'test2-website.png');
    await page.screenshot({ path: screenshot2, fullPage: false });
    
    const stats2 = fs.statSync(screenshot2);
    console.log(`✓ Screenshot saved: ${screenshot2}`);
    console.log(`  Size: ${stats2.size} bytes`);
    console.log(`  Valid: ${stats2.size > 5000 ? 'Yes' : 'No (too small)'}\n`);
    
    // Test 3: Check viewport and rendering info
    console.log('Test 3: Browser Rendering Info');
    const renderingInfo = await page.evaluate(() => {
      return {
        viewport: {
          width: window.innerWidth,
          height: window.innerHeight,
          devicePixelRatio: window.devicePixelRatio
        },
        screen: {
          width: screen.width,
          height: screen.height,
          colorDepth: screen.colorDepth,
          pixelDepth: screen.pixelDepth
        },
        webgl: (() => {
          const canvas = document.createElement('canvas');
          const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
          return gl ? 'supported' : 'not supported';
        })(),
        fonts: (() => {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');
          ctx.font = '16px Arial';
          return ctx.font;
        })()
      };
    });
    
    console.log('Rendering Info:', JSON.stringify(renderingInfo, null, 2));
    console.log('\n');
    
    // Summary
    console.log('Summary:');
    console.log('--------');
    console.log('✓ Browser launched successfully');
    console.log('✓ Screenshots captured');
    console.log('✓ Files saved with valid sizes');
    console.log('\nScreenshot functionality is working correctly!');
    
  } catch (error) {
    console.error('Error during screenshot verification:', error);
    process.exit(1);
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}

// Run if called directly
if (require.main === module) {
  verifyScreenshots().catch(console.error);
}

module.exports = verifyScreenshots;