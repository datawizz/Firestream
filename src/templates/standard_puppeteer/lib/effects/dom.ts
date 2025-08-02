// DOM Manipulation Effects

import { BaseEffect, EffectError, ExecutionContext } from './index';
import {
  NavigateParams,
  ClickParams,
  TypeParams,
  WaitForSelectorParams,
  ExtractAllParams,
  ExtractTextParams,
  ScreenshotParams,
  ExtractionRule
} from '../types';
import { Page } from 'puppeteer';

export class NavigateEffect extends BaseEffect<NavigateParams, void, void> {
  async execute(_: void, context: ExecutionContext): Promise<void> {
    this.log('info', `Navigating to ${this.params.url}`);
    
    try {
      // Set up response monitoring for better error detection
      const responses: Array<{ url: string; status: number }> = [];
      const responseHandler = (response: any) => {
        responses.push({
          url: response.url(),
          status: response.status()
        });
      };
      
      context.page.on('response', responseHandler);
      
      try {
        // Navigate with multiple wait strategies
        const waitUntil = this.params.wait_until || 'networkidle2';
        const timeout = this.params.timeout || 30000;
        
        await context.page.goto(this.params.url, {
          waitUntil: waitUntil as any,
          timeout
        });
        
        // Additional checks for JavaScript-heavy sites
        if (waitUntil === 'networkidle2' || waitUntil === 'networkidle0') {
          // Wait for basic DOM to be ready
          await context.page.evaluate(() => {
            return new Promise<void>((resolve) => {
              if (document.readyState === 'complete') {
                resolve();
              } else {
                window.addEventListener('load', () => resolve(), { once: true });
              }
            });
          });
          
          // Check if the page has meaningful content
          const hasContent = await this.waitForContent(context.page, 5000);
          if (!hasContent) {
            this.log('warn', 'Page loaded but appears to have no meaningful content');
          }
        }
        
        // Check for common error patterns
        const mainResponse = responses.find(r => r.url === this.params.url);
        if (mainResponse && mainResponse.status >= 400) {
          throw new Error(`Navigation resulted in HTTP ${mainResponse.status}`);
        }
        
      } finally {
        context.page.removeListener('response', responseHandler);
      }
      
      this.log('info', `Successfully navigated to ${this.params.url}`);
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to navigate to ${this.params.url}`,
        error as Error
      );
    }
  }
  
  private async waitForContent(page: Page, timeout: number): Promise<boolean> {
    try {
      await page.waitForFunction(
        () => {
          // Check if body has meaningful content
          const body = document.body;
          if (!body) return false;
          
          // Check text content length
          const textContent = body.innerText || body.textContent || '';
          if (textContent.trim().length < 50) return false;
          
          // Check for common loading indicators
          const loadingSelectors = [
            '.loading', '.spinner', '.loader',
            '[data-loading="true"]', '[aria-busy="true"]'
          ];
          
          for (const selector of loadingSelectors) {
            if (document.querySelector(selector)) {
              return false;
            }
          }
          
          // Check if major frameworks are still initializing
          const win = window as any;
          if (win.React && win.React.__isDevelopment) {
            // React dev mode - check if root is mounted
            const root = document.getElementById('root') || document.getElementById('app');
            if (root && root.children.length === 0) return false;
          }
          
          return true;
        },
        { timeout }
      );
      return true;
    } catch {
      return false;
    }
  }
}

export class ClickEffect extends BaseEffect<ClickParams, void, void> {
  async execute(_: void, context: ExecutionContext): Promise<void> {
    this.log('info', `Clicking element: ${this.params.selector}`);
    
    try {
      // Wait for element to be clickable
      await context.page.waitForSelector(this.params.selector, {
        visible: true,
        timeout: 10000
      });
      
      // Ensure element is actually clickable (not covered, disabled, etc.)
      const isClickable = await context.page.evaluate((selector) => {
        const element = document.querySelector(selector) as HTMLElement;
        if (!element) return false;
        
        // Check if element is visible
        const rect = element.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return false;
        
        // Check if element is disabled
        if (element.hasAttribute('disabled') || element.getAttribute('aria-disabled') === 'true') {
          return false;
        }
        
        // Check if element is covered by another element
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        const topElement = document.elementFromPoint(centerX, centerY);
        
        // Element is clickable if it's the top element or contains the top element
        return element === topElement || element.contains(topElement);
      }, this.params.selector);
      
      if (!isClickable) {
        throw new Error(`Element ${this.params.selector} is not clickable (may be covered or disabled)`);
      }
      
      // Get current URL before click for SPA detection
      const urlBefore = context.page.url();
      
      if (this.params.wait_for_navigation) {
        // Traditional page navigation
        await Promise.all([
          context.page.waitForNavigation({
            waitUntil: 'networkidle2',
            timeout: 30000
          }),
          context.page.click(this.params.selector, {
            clickCount: this.params.click_count || 1
          })
        ]);
      } else {
        // Click and handle potential SPA navigation
        await context.page.click(this.params.selector, {
          clickCount: this.params.click_count || 1
        });
        
        // Wait for potential SPA navigation
        try {
          await context.page.waitForFunction(
            (oldUrl) => window.location.href !== oldUrl,
            { timeout: 5000 },
            urlBefore
          );
          
          // URL changed, wait for content to stabilize
          this.log('info', 'Detected SPA navigation after click');
          await this.waitForSPANavigation(context.page);
        } catch {
          // No URL change, but might still have DOM updates
          await context.page.waitForTimeout(500);
        }
      }
      
      this.log('info', `Successfully clicked ${this.params.selector}`);
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to click element: ${this.params.selector}`,
        error as Error
      );
    }
  }
  
  private async waitForSPANavigation(page: Page): Promise<void> {
    try {
      // Wait for common SPA patterns
      await page.evaluate(() => {
        return new Promise<void>((resolve) => {
          let resolved = false;
          const timeout = setTimeout(() => {
            if (!resolved) {
              resolved = true;
              resolve();
            }
          }, 3000);
          
          // Listen for history changes
          const originalPushState = window.history.pushState;
          window.history.pushState = function(...args) {
            originalPushState.apply(window.history, args);
            if (!resolved) {
              resolved = true;
              clearTimeout(timeout);
              window.history.pushState = originalPushState;
              setTimeout(() => resolve(), 100);
            }
          };
          
          // Also check for DOM stability
          let lastMutation = Date.now();
          const observer = new MutationObserver(() => {
            lastMutation = Date.now();
          });
          
          observer.observe(document.body, {
            childList: true,
            subtree: true
          });
          
          // Check periodically if DOM is stable
          const checkStability = setInterval(() => {
            if (Date.now() - lastMutation > 500 && !resolved) {
              resolved = true;
              clearInterval(checkStability);
              clearTimeout(timeout);
              observer.disconnect();
              window.history.pushState = originalPushState;
              resolve();
            }
          }, 100);
        });
      });
    } catch (error) {
      this.log('warn', `SPA navigation wait failed: ${(error as Error).message}`);
    }
  }
}

export class TypeEffect extends BaseEffect<TypeParams, void, void> {
  async execute(_: void, context: ExecutionContext): Promise<void> {
    this.log('info', `Typing into ${this.params.fields.length} fields`);
    
    try {
      for (const field of this.params.fields) {
        await context.page.waitForSelector(field.selector, {
          visible: true,
          timeout: 10000
        });
        
        if (field.clear_first) {
          await context.page.click(field.selector, { clickCount: 3 });
          await context.page.keyboard.press('Backspace');
        }
        
        await context.page.type(field.selector, field.value, {
          delay: this.params.delay || 50
        });
        
        this.log('debug', `Typed into ${field.selector}`);
      }
      
      this.log('info', 'Successfully typed into all fields');
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to type into fields',
        error as Error
      );
    }
  }
}

export class WaitForSelectorEffect extends BaseEffect<WaitForSelectorParams, void, void> {
  async execute(_: void, context: ExecutionContext): Promise<void> {
    this.log('info', `Waiting for selector: ${this.params.selector}`);
    
    try {
      const options: any = {
        timeout: this.params.timeout || 30000
      };
      
      if (this.params.visible !== undefined) {
        options.visible = this.params.visible;
      }
      
      await context.page.waitForSelector(this.params.selector, options);
      
      this.log('info', `Found selector: ${this.params.selector}`);
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Timeout waiting for selector: ${this.params.selector}`,
        error as Error
      );
    }
  }
}

export class ExtractAllEffect extends BaseEffect<ExtractAllParams, void, any[]> {
  async execute(_: void, context: ExecutionContext): Promise<any[]> {
    this.log('info', `Extracting data from ${this.params.selector}`);
    
    try {
      let allData: any[] = [];
      let currentPage = 1;
      const maxPages = this.params.pagination?.max_pages || 1;
      
      while (currentPage <= maxPages) {
        // Wait for items to load
        await context.page.waitForSelector(this.params.selector, {
          timeout: 10000
        });
        
        // Extract data from current page
        const pageData = await this.extractFromPage(context.page);
        allData = allData.concat(pageData);
        
        this.log('info', `Extracted ${pageData.length} items from page ${currentPage}`);
        
        // Handle pagination
        if (this.params.pagination && currentPage < maxPages) {
          const hasNext = await this.handlePagination(context.page);
          if (!hasNext) {
            this.log('info', 'No more pages found');
            break;
          }
          currentPage++;
          
          // Add delay between pages
          const delayMs = this.params.pagination?.delay_ms;
          if (delayMs) {
            await new Promise(resolve => setTimeout(resolve, delayMs));
          }
        } else {
          break;
        }
      }
      
      this.log('info', `Extracted total of ${allData.length} items`);
      return allData;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to extract data',
        error as Error
      );
    }
  }
  
  private async extractFromPage(page: Page): Promise<any[]> {
    return await page.evaluate((selector, fields) => {
      const elements = document.querySelectorAll(selector);
      
      return Array.from(elements).map(element => {
        const data: any = {};
        
        for (const [fieldName, rule] of Object.entries(fields)) {
          const fieldRule = rule as ExtractionRule;
          const fieldElement = element.querySelector(fieldRule.selector);
          
          if (!fieldElement) {
            data[fieldName] = fieldRule.default_value || null;
            continue;
          }
          
          let value: any;
          switch (fieldRule.extraction_type) {
            case 'text':
              value = fieldElement.textContent?.trim() || '';
              break;
            case 'attribute':
              value = fieldElement.getAttribute(fieldRule.attribute || '');
              break;
            case 'html':
              value = fieldElement.innerHTML;
              break;
            case 'value':
              value = (fieldElement as HTMLInputElement).value;
              break;
            default:
              value = fieldElement.textContent?.trim() || '';
          }
          
          // Apply transform if specified
          if (fieldRule.transform && value) {
            try {
              // Safe evaluation of simple transforms
              if (fieldRule.transform === 'parseFloat') {
                value = parseFloat(value.toString().replace(/[^0-9.-]/g, ''));
              } else if (fieldRule.transform === 'parseInt') {
                value = parseInt(value.toString().replace(/[^0-9-]/g, ''));
              } else if (fieldRule.transform.startsWith('text => ')) {
                // Handle simple inline functions
                const func = new Function('text', fieldRule.transform.replace('text => ', 'return '));
                value = func(value);
              }
            } catch (e) {
              console.warn(`Transform failed for field ${fieldName}:`, e);
            }
          }
          
          data[fieldName] = value;
        }
        
        return data;
      });
    }, this.params.selector, this.params.fields);
  }
  
  private async handlePagination(page: Page): Promise<boolean> {
    if (!this.params.pagination) return false;
    
    switch (this.params.pagination.pagination_type) {
      case 'click_next':
        if (!this.params.pagination.next_selector) return false;
        
        try {
          const nextButton = await page.$(this.params.pagination.next_selector);
          if (!nextButton) return false;
          
          const isDisabled = await page.evaluate(
            el => el.hasAttribute('disabled') || el.classList.contains('disabled'),
            nextButton
          );
          
          if (isDisabled) return false;
          
          await nextButton.click();
          // Wait for navigation to complete
          await page.waitForNavigation({ waitUntil: 'networkidle2' });
          return true;
        } catch {
          return false;
        }
        
      case 'infinite_scroll':
        // Scroll to bottom and wait for new content
        const previousHeight = await page.evaluate(() => document.body.scrollHeight);
        await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
        await new Promise(resolve => setTimeout(resolve, 2000));
        const newHeight = await page.evaluate(() => document.body.scrollHeight);
        return newHeight > previousHeight;
        
      default:
        return false;
    }
  }
}

export class ExtractTextEffect extends BaseEffect<ExtractTextParams, void, string | string[]> {
  async execute(_: void, context: ExecutionContext): Promise<string | string[]> {
    this.log('info', `Extracting text from ${this.params.selector}`);
    
    try {
      if (this.params.multiple) {
        const texts = await context.page.$$eval(
          this.params.selector,
          elements => elements.map(el => el.textContent?.trim() || '')
        );
        this.log('info', `Extracted ${texts.length} text values`);
        return texts;
      } else {
        const text = await context.page.$eval(
          this.params.selector,
          el => el.textContent?.trim() || ''
        );
        this.log('info', `Extracted text: ${text.substring(0, 50)}...`);
        return text;
      }
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to extract text from ${this.params.selector}`,
        error as Error
      );
    }
  }
}

export class ScreenshotEffect extends BaseEffect<ScreenshotParams, void, string> {
  async execute(_: void, context: ExecutionContext): Promise<string> {
    this.log('info', `Taking screenshot: ${this.params.filename}`);
    
    try {
      // Ensure page is in a stable state before screenshot
      await this.ensurePageReady(context.page);
      
      const screenshotPath = `screenshots/${context.workflowId}/${this.params.filename}`;
      
      // Create directory if it doesn't exist
      const fs = await import('fs');
      const path = await import('path');
      const dir = path.dirname(screenshotPath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      
      // Take screenshot with retry logic
      let screenshot: Buffer | undefined;
      let attempts = 0;
      const maxAttempts = 3;
      
      while (attempts < maxAttempts && !screenshot) {
        attempts++;
        try {
          screenshot = await context.page.screenshot({
            path: screenshotPath,
            fullPage: this.params.full_page || false,
            type: 'png',
            captureBeyondViewport: true
          });
          
          // Verify screenshot is not blank (has reasonable size)
          if (screenshot && screenshot.length < 5000) {
            this.log('warn', `Screenshot appears blank (${screenshot.length} bytes), retrying...`);
            screenshot = undefined;
            await new Promise(resolve => setTimeout(resolve, 2000));
          }
        } catch (err) {
          this.log('warn', `Screenshot attempt ${attempts} failed: ${(err as Error).message}`);
          if (attempts < maxAttempts) {
            await new Promise(resolve => setTimeout(resolve, 1000));
          }
        }
      }
      
      if (!screenshot) {
        throw new Error('Failed to capture non-blank screenshot after multiple attempts');
      }
      
      this.log('info', `Screenshot saved to ${screenshotPath}`);
      
      // Optionally upload to S3
      if (this.params.upload_to_s3) {
        // This would be handled by a separate S3 upload effect
        context.data.set(`${this.id}_screenshot_path`, screenshotPath);
      }
      
      return screenshotPath;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to take screenshot',
        error as Error
      );
    }
  }
  
  private async ensurePageReady(page: Page): Promise<void> {
    try {
      // Wait for page to be fully loaded
      await page.evaluate(() => {
        return new Promise<void>((resolve) => {
          if (document.readyState === 'complete') {
            resolve();
          } else {
            window.addEventListener('load', () => resolve(), { once: true });
            // Timeout after 5 seconds
            setTimeout(() => resolve(), 5000);
          }
        });
      });
      
      // Wait for any pending animations
      await page.evaluate(() => {
        return new Promise<void>((resolve) => {
          if (typeof window.requestAnimationFrame === 'function') {
            window.requestAnimationFrame(() => {
              window.requestAnimationFrame(() => resolve());
            });
          } else {
            setTimeout(() => resolve(), 100);
          }
        });
      });
      
      // Additional wait for dynamic content
      await page.waitForTimeout(500);
    } catch (error) {
      this.log('warn', `Page readiness check failed: ${(error as Error).message}`);
      // Continue anyway - better to get some screenshot than none
    }
  }
}