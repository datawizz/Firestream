// Cookie Management Effects

import { BaseEffect, EffectError, ExecutionContext } from './index';
import { ExtractCookiesParams, SetCookieParams, ClearCookiesParams } from '../types';
import { Protocol } from 'puppeteer';

export interface CookieData {
  name: string;
  value: string;
  domain: string;
  path: string;
  expires?: number;
  size?: number;
  httpOnly?: boolean;
  secure?: boolean;
  session?: boolean;
  sameSite?: 'Strict' | 'Lax' | 'None';
}

export class ExtractCookiesEffect extends BaseEffect<ExtractCookiesParams, void, CookieData[]> {
  async execute(_: void, context: ExecutionContext): Promise<CookieData[]> {
    this.log('info', 'Extracting cookies from browser');
    
    try {
      // Get all cookies from the current page
      const cookies = await context.page.cookies();
      
      this.log('debug', `Found ${cookies.length} cookies in browser`, {
        total: cookies.length,
        domains: [...new Set(cookies.map(c => c.domain))]
      });
      
      let filteredCookies = cookies;
      
      // Apply domain filter if specified
      if (this.params.domain_filter) {
        filteredCookies = filteredCookies.filter(cookie => 
          cookie.domain.includes(this.params.domain_filter!)
        );
        this.log('debug', `Filtered to ${filteredCookies.length} cookies for domain: ${this.params.domain_filter}`);
      }
      
      // Apply cookie name filter if specified
      if (this.params.cookie_names && this.params.cookie_names.length > 0 && !this.params.include_all) {
        filteredCookies = filteredCookies.filter(cookie =>
          this.params.cookie_names!.includes(cookie.name)
        );
        this.log('debug', `Filtered to ${filteredCookies.length} cookies by names: ${this.params.cookie_names.join(', ')}`);
      }
      
      // Transform to our cookie format
      const cookieData: CookieData[] = filteredCookies.map(cookie => ({
        name: cookie.name,
        value: cookie.value,
        domain: cookie.domain,
        path: cookie.path,
        expires: cookie.expires,
        size: cookie.size,
        httpOnly: cookie.httpOnly,
        secure: cookie.secure,
        session: cookie.session,
        sameSite: cookie.sameSite as 'Strict' | 'Lax' | 'None'
      }));
      
      this.log('info', `Successfully extracted ${cookieData.length} cookies`, {
        cookies: cookieData.map(c => ({ name: c.name, domain: c.domain, size: c.size }))
      });
      
      // Store cookies in context for use by other effects
      context.data.set(`${this.id}_cookies`, cookieData);
      context.data.set(`${this.id}_cookie_count`, cookieData.length);
      
      // Store as HTTP Cookie header format for API requests
      const cookieHeader = cookieData
        .map(c => `${c.name}=${c.value}`)
        .join('; ');
      
      if (cookieHeader) {
        context.data.set(`${this.id}_cookie_header`, cookieHeader);
        
        // Update auth context with cookie headers
        context.auth = context.auth || {};
        context.auth.headers = context.auth.headers || {};
        context.auth.headers['Cookie'] = cookieHeader;
        
        this.log('debug', 'Updated auth context with cookie header', {
          headerLength: cookieHeader.length,
          cookieCount: cookieData.length
        });
      }
      
      return cookieData;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to extract cookies: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (this.params.cookie_names && this.params.cookie_names.length === 0) {
      throw new Error('Cookie names array cannot be empty if specified');
    }
    
    if (this.params.cookie_names && this.params.include_all) {
      this.log('warn', 'Both cookie_names and include_all specified. include_all will take precedence.');
    }
  }
}

export class SetCookieEffect extends BaseEffect<SetCookieParams, void, void> {
  async execute(_: void, context: ExecutionContext): Promise<void> {
    this.log('info', `Setting cookie: ${this.params.name}=${this.params.value.substring(0, 20)}...`);
    
    try {
      const cookieOptions: Protocol.Network.CookieParam = {
        name: this.params.name,
        value: this.params.value,
        path: this.params.path || '/',
        secure: this.params.secure || false,
        httpOnly: this.params.httpOnly || false
      };
      
      // Add domain only if provided
      if (this.params.domain) {
        cookieOptions.domain = this.params.domain;
      }
      
      if (this.params.maxAge) {
        cookieOptions.expires = Math.floor(Date.now() / 1000) + this.params.maxAge;
      }
      
      await context.page.setCookie(cookieOptions);
      
      this.log('info', 'Successfully set cookie', {
        name: this.params.name,
        domain: cookieOptions.domain,
        path: cookieOptions.path,
        secure: cookieOptions.secure,
        httpOnly: cookieOptions.httpOnly
      });
      
      // Store cookie info in context
      context.data.set(`${this.id}_cookie_set`, {
        name: this.params.name,
        domain: cookieOptions.domain,
        timestamp: new Date().toISOString()
      });
      
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to set cookie: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (!this.params.name || !this.params.value) {
      throw new Error('Cookie name and value are required');
    }
    
    if (this.params.name.includes('=') || this.params.name.includes(';')) {
      throw new Error('Cookie name cannot contain = or ; characters');
    }
  }
}

export class ClearCookiesEffect extends BaseEffect<ClearCookiesParams, void, number> {
  async execute(_: void, context: ExecutionContext): Promise<number> {
    this.log('info', 'Clearing cookies from browser');
    
    try {
      const currentCookies = await context.page.cookies();
      let cookiesToClear = currentCookies;
      
      // Apply domain filter if specified
      if (this.params.domain_filter) {
        cookiesToClear = cookiesToClear.filter(cookie => 
          cookie.domain.includes(this.params.domain_filter!)
        );
        this.log('debug', `Filtered to ${cookiesToClear.length} cookies for domain: ${this.params.domain_filter}`);
      }
      
      // Apply cookie name filter if specified
      if (this.params.cookie_names && this.params.cookie_names.length > 0 && !this.params.clear_all) {
        cookiesToClear = cookiesToClear.filter(cookie =>
          this.params.cookie_names!.includes(cookie.name)
        );
        this.log('debug', `Filtered to ${cookiesToClear.length} cookies by names: ${this.params.cookie_names.join(', ')}`);
      }
      
      // Clear the cookies
      let clearedCount = 0;
      for (const cookie of cookiesToClear) {
        try {
          await context.page.deleteCookie({
            name: cookie.name,
            domain: cookie.domain,
            path: cookie.path
          });
          clearedCount++;
        } catch (error) {
          this.log('warn', `Failed to clear cookie ${cookie.name}`, { error: error instanceof Error ? error.message : 'Unknown error' });
        }
      }
      
      this.log('info', `Successfully cleared ${clearedCount} cookies`, {
        requested: cookiesToClear.length,
        cleared: clearedCount
      });
      
      // Store clear operation info in context
      context.data.set(`${this.id}_cookies_cleared`, clearedCount);
      context.data.set(`${this.id}_clear_timestamp`, new Date().toISOString());
      
      return clearedCount;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to clear cookies: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (this.params.cookie_names && this.params.cookie_names.length === 0) {
      throw new Error('Cookie names array cannot be empty if specified');
    }
    
    if (this.params.cookie_names && this.params.clear_all) {
      this.log('warn', 'Both cookie_names and clear_all specified. clear_all will take precedence.');
    }
    
    if (!this.params.cookie_names && !this.params.clear_all && !this.params.domain_filter) {
      throw new Error('Must specify either cookie_names, clear_all, or domain_filter');
    }
  }
}