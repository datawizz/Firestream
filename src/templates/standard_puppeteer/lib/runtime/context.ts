// Execution Context Management

import { Browser, Page } from 'puppeteer';
import { logger } from '../utils/logger';
import { ExecutionContext } from '../effects';

export interface ContextOptions {
  workflowId: string;
  executionId: string;
  browser: Browser;
  page: Page;
  initialData?: Record<string, any>;
}

export class ContextManager {
  private contexts: Map<string, ExecutionContext> = new Map();
  
  createContext(options: ContextOptions): ExecutionContext {
    const context: ExecutionContext = {
      browser: options.browser,
      page: options.page,
      data: new Map(Object.entries(options.initialData || {})),
      logger,
      workflowId: options.workflowId,
      executionId: options.executionId
      // auth is optional, so we don't set it initially
    };
    
    this.contexts.set(options.executionId, context);
    
    logger.debug('Execution context created', {
      workflowId: options.workflowId,
      executionId: options.executionId,
      initialDataKeys: Object.keys(options.initialData || {})
    });
    
    return context;
  }
  
  getContext(executionId: string): ExecutionContext | undefined {
    return this.contexts.get(executionId);
  }
  
  updateAuth(executionId: string, auth: { token?: string; headers?: Record<string, string> }): void {
    const context = this.contexts.get(executionId);
    if (context) {
      context.auth = { ...context.auth, ...auth };
      logger.debug('Context auth updated', { 
        executionId, 
        hasToken: !!auth.token,
        headerCount: Object.keys(auth.headers || {}).length
      });
    }
  }
  
  setData(executionId: string, key: string, value: any): void {
    const context = this.contexts.get(executionId);
    if (context) {
      context.data.set(key, value);
      logger.debug('Context data updated', { executionId, key, valueType: typeof value });
    }
  }
  
  getData(executionId: string, key: string): any {
    const context = this.contexts.get(executionId);
    return context?.data.get(key);
  }
  
  getAllData(executionId: string): Record<string, any> {
    const context = this.contexts.get(executionId);
    if (!context) return {};
    
    const data: Record<string, any> = {};
    for (const [key, value] of context.data.entries()) {
      data[key] = value;
    }
    return data;
  }
  
  removeContext(executionId: string): void {
    this.contexts.delete(executionId);
    logger.debug('Execution context removed', { executionId });
  }
  
  clearAll(): void {
    this.contexts.clear();
    logger.debug('All execution contexts cleared');
  }
}

// Global context manager instance
export const contextManager = new ContextManager();

// Helper functions for context manipulation
export function mergeContextData(
  context: ExecutionContext,
  data: Record<string, any>
): void {
  for (const [key, value] of Object.entries(data)) {
    context.data.set(key, value);
  }
}

export function extractContextData(
  context: ExecutionContext,
  keys: string[]
): Record<string, any> {
  const result: Record<string, any> = {};
  for (const key of keys) {
    if (context.data.has(key)) {
      result[key] = context.data.get(key);
    }
  }
  return result;
}

// Context data serialization for debugging/logging
export function serializeContextData(context: ExecutionContext): string {
  const data: Record<string, any> = {
    workflowId: context.workflowId,
    executionId: context.executionId,
    hasAuth: !!context.auth,
    dataKeys: Array.from(context.data.keys()),
    dataSummary: {}
  };
  
  // Include summary of data values (truncated for large values)
  for (const [key, value] of context.data.entries()) {
    if (typeof value === 'string' && value.length > 100) {
      data['dataSummary'][key] = `${value.substring(0, 100)}... (${value.length} chars)`;
    } else if (Array.isArray(value)) {
      data['dataSummary'][key] = `Array(${value.length})`;
    } else if (typeof value === 'object' && value !== null) {
      data['dataSummary'][key] = `Object(${Object.keys(value).length} keys)`;
    } else {
      data['dataSummary'][key] = value;
    }
  }
  
  return JSON.stringify(data, null, 2);
}