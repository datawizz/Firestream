const path = require('path');
const fs = require('fs');

class ScreenshotHelper {
  constructor(page, config = {}) {
    this.page = page;
    this.config = {
      baseDir: config.baseDir || path.join(__dirname, '../../results/screenshots'),
      testName: config.testName || 'test',
      timestamp: config.timestamp || Date.now(),
      ...config
    };
    
    this.screenshots = [];
    this.ensureDirectoryExists();
  }

  ensureDirectoryExists() {
    const testDir = path.join(this.config.baseDir, `${this.config.testName}_${this.config.timestamp}`);
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }
    this.testDir = testDir;
  }

  async capture(stepName, options = {}) {
    const maxRetries = options.retries || 3;
    const waitBeforeScreenshot = options.wait || 500;
    
    // Generate filename once for all attempts
    const filename = this.generateFilename(stepName);
    const filepath = path.join(this.testDir, filename);
    
    // Wait for any pending animations or renders
    if (waitBeforeScreenshot > 0) {
      await this.page.waitForTimeout(waitBeforeScreenshot);
    }
    
    // Wait for network idle if specified
    if (options.waitForNetworkIdle) {
      try {
        await this.page.waitForLoadState('networkidle', { timeout: 5000 });
      } catch (e) {
        console.warn('Network idle timeout, proceeding with screenshot');
      }
    }
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        
        // Ensure page is in a stable state
        await this.page.evaluate(() => {
          // Force any pending renders
          if (window.requestAnimationFrame) {
            return new Promise(resolve => window.requestAnimationFrame(resolve));
          }
        });
        
        // Take screenshot with options
        const screenshotOptions = {
          path: filepath,
          fullPage: options.fullPage !== false,
          type: 'png',
          quality: undefined, // PNG doesn't use quality
          clip: options.clip,
          omitBackground: false,
          captureBeyondViewport: true,
          ...options
        };
        
        // Remove quality if PNG
        if (screenshotOptions.type === 'png') {
          delete screenshotOptions.quality;
        }
        
        await this.page.screenshot(screenshotOptions);
        
        // Verify the screenshot was saved and has content
        if (fs.existsSync(filepath)) {
          const stats = fs.statSync(filepath);
          if (stats.size > 1000) { // Ensure it's not a blank/corrupt image
            // Store screenshot info
            this.screenshots.push({
              step: stepName,
              filename,
              filepath,
              timestamp: new Date().toISOString(),
              size: stats.size,
              viewport: this.page.viewport()
            });
            
            console.log(`Screenshot captured: ${filename} (${stats.size} bytes)`);
            return filepath;
          } else {
            console.warn(`Screenshot ${filename} appears to be empty (${stats.size} bytes), retrying...`);
            fs.unlinkSync(filepath);
          }
        }
      } catch (error) {
        console.error(`Screenshot attempt ${attempt}/${maxRetries} failed for ${stepName}:`, error.message);
        if (attempt < maxRetries) {
          await this.page.waitForTimeout(1000); // Wait before retry
        }
      }
    }
    
    console.error(`Failed to capture screenshot for ${stepName} after ${maxRetries} attempts`);
    return null;
  }

  async captureElement(selector, stepName, options = {}) {
    try {
      const element = await this.page.$(selector);
      if (!element) {
        console.warn(`Element not found for selector: ${selector}`);
        return null;
      }
      
      const filename = this.generateFilename(stepName);
      const filepath = path.join(this.testDir, filename);
      
      await element.screenshot({ path: filepath, ...options });
      
      this.screenshots.push({
        step: stepName,
        selector,
        filename,
        filepath,
        timestamp: new Date().toISOString()
      });
      
      console.log(`Element screenshot captured: ${filename}`);
      return filepath;
    } catch (error) {
      console.error(`Failed to capture element screenshot for ${stepName}:`, error);
      return null;
    }
  }

  async captureOnError(error, stepName = 'error') {
    try {
      // First, ensure the page is in a stable state
      await this.page.evaluate(() => {
        // Check if document is ready
        if (document.readyState !== 'complete') {
          return new Promise(resolve => {
            window.addEventListener('load', resolve, { once: true });
            // Timeout after 5 seconds
            setTimeout(resolve, 5000);
          });
        }
      });
      
      // Wait a bit for any error states to render
      await this.page.waitForTimeout(1000);
      
      // Check if page is blank and try to gather diagnostic info
      const pageInfo = await this.page.evaluate(() => {
        return {
          url: window.location.href,
          title: document.title,
          bodyHtml: document.body ? document.body.innerHTML.substring(0, 500) : 'No body element',
          bodyText: document.body ? document.body.innerText.substring(0, 500) : 'No body text',
          hasContent: document.body && document.body.children.length > 0,
          readyState: document.readyState,
          documentElement: document.documentElement ? document.documentElement.outerHTML.substring(0, 200) : 'No document element'
        };
      });
      
      console.log('Page state at error:', JSON.stringify(pageInfo, null, 2));
      
      // If page appears blank, wait longer for content to load
      if (!pageInfo.hasContent || pageInfo.bodyText.trim().length < 10) {
        console.log('Page appears blank, waiting for content...');
        try {
          await this.page.waitForSelector('body > *', { timeout: 5000 });
        } catch (e) {
          console.log('Timeout waiting for page content');
        }
      }
    } catch (diagnosticError) {
      console.error('Error gathering diagnostic info:', diagnosticError.message);
    }
    
    const errorScreenshot = await this.capture(`${stepName}_error`, {
      fullPage: true,
      wait: 2000, // Wait 2 seconds before screenshot
      waitForNetworkIdle: true
    });
    
    // Also capture console logs if available
    try {
      const consoleLogs = await this.page.evaluate(() => {
        return window.__consoleLogs || [];
      });
      
      if (consoleLogs.length > 0) {
        const logPath = path.join(this.testDir, `${stepName}_console.log`);
        fs.writeFileSync(logPath, JSON.stringify(consoleLogs, null, 2));
      }
    } catch (e) {
      // Ignore console log capture errors
    }
    
    // Save page HTML for debugging
    try {
      const html = await this.page.content();
      const htmlPath = path.join(this.testDir, `${stepName}_error.html`);
      fs.writeFileSync(htmlPath, html);
    } catch (e) {
      console.error('Failed to save error HTML:', e.message);
    }
    
    return errorScreenshot;
  }

  async captureSequence(steps) {
    const results = [];
    
    for (const step of steps) {
      const { name, action, selector, wait } = step;
      
      // Perform action if provided
      if (action) {
        try {
          await action();
        } catch (error) {
          console.error(`Action failed for ${name}:`, error);
          await this.captureOnError(error, name);
          continue;
        }
      }
      
      // Wait if specified
      if (wait) {
        await this.page.waitForTimeout(wait);
      }
      
      // Capture screenshot
      const screenshot = selector 
        ? await this.captureElement(selector, name)
        : await this.capture(name);
        
      results.push({
        step: name,
        screenshot,
        success: !!screenshot
      });
    }
    
    return results;
  }

  generateFilename(stepName) {
    const sanitized = stepName.replace(/[^a-zA-Z0-9-_]/g, '_');
    const timestamp = new Date().getTime();
    return `${this.config.testName}_${sanitized}_${timestamp}.png`;
  }

  getScreenshots() {
    return this.screenshots;
  }
  
  async captureWithDebugInfo(stepName, options = {}) {
    // Capture screenshot
    const screenshotPath = await this.capture(stepName, options);
    
    if (screenshotPath && options.saveHtml !== false) {
      try {
        // Save HTML content
        const htmlContent = await this.page.content();
        const htmlPath = screenshotPath.replace('.png', '.html');
        fs.writeFileSync(htmlPath, htmlContent);
        
        // Save page metadata
        const metadata = {
          url: this.page.url(),
          title: await this.page.title(),
          viewport: this.page.viewport(),
          timestamp: new Date().toISOString(),
          userAgent: await this.page.evaluate(() => navigator.userAgent),
          consoleErrors: await this.page.evaluate(() => {
            return window.__consoleLogs?.filter(log => log.type === 'error') || [];
          })
        };
        
        const metadataPath = screenshotPath.replace('.png', '.json');
        fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));
        
        console.log(`Debug info saved: HTML and metadata for ${stepName}`);
      } catch (error) {
        console.error(`Failed to save debug info for ${stepName}:`, error);
      }
    }
    
    return screenshotPath;
  }

  async generateComparisonReport(baseline) {
    // This could be extended to do visual regression testing
    const report = {
      test: this.config.testName,
      screenshots: this.screenshots,
      baseline,
      timestamp: new Date().toISOString()
    };
    
    const reportPath = path.join(this.testDir, 'screenshot-report.json');
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    
    return reportPath;
  }

  // Helper method to annotate screenshots with text
  async captureWithAnnotation(stepName, annotationText, options = {}) {
    // Add annotation to the page
    await this.page.evaluate((text) => {
      const annotation = document.createElement('div');
      annotation.id = 'test-annotation';
      annotation.style.cssText = `
        position: fixed;
        top: 10px;
        right: 10px;
        background: rgba(255, 0, 0, 0.8);
        color: white;
        padding: 10px 20px;
        border-radius: 4px;
        font-size: 16px;
        font-weight: bold;
        z-index: 999999;
      `;
      annotation.textContent = text;
      document.body.appendChild(annotation);
    }, annotationText);
    
    // Take screenshot
    const screenshot = await this.capture(stepName, options);
    
    // Remove annotation
    await this.page.evaluate(() => {
      const annotation = document.getElementById('test-annotation');
      if (annotation) {
        annotation.remove();
      }
    });
    
    return screenshot;
  }
}

module.exports = ScreenshotHelper;