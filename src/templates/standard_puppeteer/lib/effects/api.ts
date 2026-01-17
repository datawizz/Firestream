// API Request Effects

import { BaseEffect, EffectError, ExecutionContext } from './index';
import { ApiRequestParams, HttpMethod } from '../types';
import axios, { AxiosRequestConfig, AxiosResponse } from 'axios';

export class ApiRequestEffect extends BaseEffect<ApiRequestParams, any, any> {
  async execute(_input: any, context: ExecutionContext): Promise<any> {
    this.log('info', `Making ${this.params.method} request to ${this.params.endpoint}`);
    
    try {
      const config: AxiosRequestConfig = {
        method: this.params.method.toLowerCase() as any,
        url: this.params.endpoint,
        headers: {
          'Content-Type': 'application/json',
          ...this.params.headers
        },
        params: this.params.params,
        data: this.params.data
      };
      
      // Add auth from context if requested
      if (this.params.auth_from_context && context.auth) {
        if (context.auth.token) {
          config.headers!['Authorization'] = `Bearer ${context.auth.token}`;
        }
        if (context.auth.headers) {
          Object.assign(config.headers!, context.auth.headers);
        }
      }
      
      // Add cookies from context if requested
      if (this.params.cookies_from_context) {
        let cookieHeader = '';
        
        if (this.params.cookie_effect_id) {
          // Use cookies from specific effect
          const cookieData = context.data.get(`${this.params.cookie_effect_id}_cookie_header`);
          if (cookieData) {
            cookieHeader = cookieData;
            this.log('debug', `Using cookies from effect ${this.params.cookie_effect_id}`, {
              headerLength: cookieHeader.length
            });
          } else {
            this.log('warn', `No cookie header found for effect ${this.params.cookie_effect_id}`);
          }
        } else if (context.auth && context.auth.headers && context.auth.headers['Cookie']) {
          // Use cookies from auth context
          cookieHeader = context.auth.headers['Cookie'];
          this.log('debug', 'Using cookies from auth context', {
            headerLength: cookieHeader.length
          });
        }
        
        if (cookieHeader) {
          config.headers!['Cookie'] = cookieHeader;
        } else {
          this.log('warn', 'cookies_from_context requested but no cookies found in context');
        }
      }
      
      this.log('debug', 'Request config', { 
        method: config.method,
        url: config.url,
        headers: Object.keys(config.headers || {}),
        hasParams: !!config.params,
        hasData: !!config.data
      });
      
      let response: any;
      
      // Check if we're running in a browser context (Puppeteer)
      if (context.page) {
        this.log('debug', 'Using browser context for API request');
        
        // Execute request in browser context to use browser cookies
        const browserResponse = await context.page.evaluate(async (requestConfig) => {
          try {
            // Build URL with query parameters
            let url = requestConfig.url || '';
            if (requestConfig.params) {
              const params = new URLSearchParams(requestConfig.params);
              url += (url.includes('?') ? '&' : '?') + params.toString();
            }
            
            const fetchOptions: RequestInit = {
              method: requestConfig.method || 'GET',
              credentials: 'include' // Important: include cookies
            };
            
            if (requestConfig.headers) {
              fetchOptions.headers = Object.fromEntries(
                Object.entries(requestConfig.headers).map(([k, v]) => [k, String(v)])
              );
            }
            
            if (requestConfig.data) {
              fetchOptions.body = JSON.stringify(requestConfig.data);
            }
            
            const fetchResponse = await fetch(url, fetchOptions);
            
            const responseData = await fetchResponse.json().catch(() => fetchResponse.text());
            
            // Convert headers to plain object
            const headers: Record<string, string> = {};
            fetchResponse.headers.forEach((value, key) => {
              headers[key] = value;
            });
            
            return {
              status: fetchResponse.status,
              statusText: fetchResponse.statusText,
              data: responseData,
              headers: headers,
              ok: fetchResponse.ok
            };
          } catch (error: any) {
            return {
              error: true,
              message: error.message,
              status: 0
            };
          }
        }, config);
        
        if (browserResponse.error) {
          throw new Error(`Browser fetch failed: ${browserResponse.message}`);
        }
        
        response = {
          status: browserResponse.status,
          statusText: browserResponse.statusText,
          data: browserResponse.data,
          headers: browserResponse.headers
        };
      } else {
        // Fallback to axios for non-browser execution
        this.log('debug', 'Using axios for API request (no browser context)');
        const axiosResponse: AxiosResponse = await axios(config);
        response = axiosResponse;
      }
      
      this.log('info', `Request successful: ${response.status} ${response.statusText}`);
      this.log('debug', 'Response headers', response.headers);
      
      // Store response data for subsequent effects
      context.data.set(`${this.id}_response`, response.data);
      context.data.set(`${this.id}_status`, response.status);
      context.data.set(`${this.id}_headers`, response.headers);
      
      return response.data;
    } catch (error: any) {
      if (error.response) {
        // Request made and server responded with error status
        this.log('error', `API request failed: ${error.response.status} ${error.response.statusText}`, {
          data: error.response.data,
          headers: error.response.headers
        });
        
        throw new EffectError(
          this.id,
          this.type,
          `API request failed: ${error.response.status} ${error.response.statusText}`,
          error
        );
      } else if (error.request) {
        // Request made but no response received
        this.log('error', 'No response received from API', { request: error.request });
        
        throw new EffectError(
          this.id,
          this.type,
          'No response received from API',
          error
        );
      } else {
        // Something else happened
        this.log('error', 'API request setup failed', { message: error.message });
        
        throw new EffectError(
          this.id,
          this.type,
          `API request failed: ${error.message}`,
          error
        );
      }
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (!this.params.endpoint) {
      throw new Error('API endpoint is required');
    }
    
    const validMethods: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'];
    if (!validMethods.includes(this.params.method)) {
      throw new Error(`Invalid HTTP method: ${this.params.method}`);
    }
    
    if (this.params.cookies_from_context && this.params.cookie_effect_id) {
      if (typeof this.params.cookie_effect_id !== 'string' || this.params.cookie_effect_id.trim() === '') {
        throw new Error('cookie_effect_id must be a non-empty string when specified');
      }
    }
  }
}

// Helper effect to extract auth tokens from page context
export class ExtractAuthTokenEffect extends BaseEffect<{ storage_type: string; token_keys: string[] }, void, string | null> {
  async execute(_: void, context: ExecutionContext): Promise<string | null> {
    this.log('info', `Extracting auth token from ${this.params.storage_type}`);
    
    try {
      let token: string | null = null;
      
      switch (this.params.storage_type) {
        case 'local_storage':
          token = await this.extractFromLocalStorage(context);
          break;
        case 'session_storage':
          token = await this.extractFromSessionStorage(context);
          break;
        case 'cookie':
          token = await this.extractFromCookies(context);
          break;
        default:
          throw new Error(`Unknown storage type: ${this.params.storage_type}`);
      }
      
      if (token) {
        this.log('info', 'Successfully extracted auth token');
        // Store in context for subsequent API requests
        context.auth = context.auth || {};
        context.auth.token = token;
      } else {
        this.log('warn', 'No auth token found');
      }
      
      return token;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to extract auth token',
        error as Error
      );
    }
  }
  
  private async extractFromLocalStorage(context: ExecutionContext): Promise<string | null> {
    return await context.page.evaluate((keys) => {
      for (const key of keys) {
        const value = localStorage.getItem(key);
        if (value) {
          try {
            // Try to parse as JSON first
            const parsed = JSON.parse(value);
            if (parsed.token) return parsed.token;
            if (parsed.access_token) return parsed.access_token;
            if (parsed.accessToken) return parsed.accessToken;
          } catch {
            // If not JSON, return as is
            return value;
          }
        }
      }
      return null;
    }, this.params.token_keys);
  }
  
  private async extractFromSessionStorage(context: ExecutionContext): Promise<string | null> {
    return await context.page.evaluate((keys) => {
      for (const key of keys) {
        const value = sessionStorage.getItem(key);
        if (value) {
          try {
            const parsed = JSON.parse(value);
            if (parsed.token) return parsed.token;
            if (parsed.access_token) return parsed.access_token;
            if (parsed.accessToken) return parsed.accessToken;
          } catch {
            return value;
          }
        }
      }
      return null;
    }, this.params.token_keys);
  }
  
  private async extractFromCookies(context: ExecutionContext): Promise<string | null> {
    const cookies = await context.page.cookies();
    
    for (const key of this.params.token_keys) {
      const cookie = cookies.find(c => c.name === key);
      if (cookie) {
        return cookie.value;
      }
    }
    
    return null;
  }
}