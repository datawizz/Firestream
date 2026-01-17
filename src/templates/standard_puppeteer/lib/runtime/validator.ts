// Workflow Validator
// Validates workflow JSON at runtime with comprehensive error messages

import { WorkflowDefinition } from '../types';

export interface ValidationError {
  path: string;
  message: string;
  value?: any;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export class WorkflowValidator {
  private errors: ValidationError[] = [];
  
  /**
   * Validate a workflow definition
   */
  validate(json: unknown): ValidationResult {
    this.errors = [];
    
    if (!this.isObject(json)) {
      this.addError('', 'Workflow must be an object');
      return this.getResult();
    }
    
    const workflow = json as any;
    
    // Validate required fields
    this.validateRequiredField(workflow, 'workflow_id', 'string');
    this.validateRequiredField(workflow, 'config', 'object');
    this.validateRequiredField(workflow, 'effects', 'array');
    
    // Version can be in metadata
    if (workflow.metadata && workflow.metadata.version) {
      workflow.version = workflow.metadata.version;
    } else if (!workflow.version) {
      workflow.version = '1.0.0'; // Default version
    }
    
    // Validate config
    if (workflow.config) {
      this.validateConfig(workflow.config);
    }
    
    // Validate effects
    if (Array.isArray(workflow.effects)) {
      this.validateEffects(workflow.effects);
    }
    
    // Validate schemas if present
    if (workflow.schemas) {
      this.validateSchemas(workflow.schemas);
    }
    
    return this.getResult();
  }
  
  /**
   * Type guard for valid workflow
   */
  isValidWorkflow(json: unknown): json is WorkflowDefinition {
    const result = this.validate(json);
    return result.valid;
  }
  
  private validateConfig(config: any): void {
    const path = 'config';
    
    // Validate rate limit
    if (config.rate_limit) {
      this.validateRateLimit(config.rate_limit, `${path}.rate_limit`);
    } else {
      this.addError(`${path}.rate_limit`, 'Rate limit configuration is required');
    }
    
    // Validate retry
    if (config.retry) {
      this.validateRetry(config.retry, `${path}.retry`);
    } else {
      this.addError(`${path}.retry`, 'Retry configuration is required');
    }
    
    // Validate S3
    if (config.s3) {
      this.validateS3(config.s3, `${path}.s3`);
    }
    
    // Validate browser
    if (config.browser) {
      this.validateBrowser(config.browser, `${path}.browser`);
    }
    
    // Validate logging
    if (config.logging) {
      this.validateLogging(config.logging, `${path}.logging`);
    }
  }
  
  private validateRateLimit(rateLimit: any, path: string): void {
    this.validateRequiredField(rateLimit, 'requests_per_second', 'number', path);
    this.validateRequiredField(rateLimit, 'burst', 'number', path);
    
    if (typeof rateLimit.requests_per_second === 'number' && rateLimit.requests_per_second <= 0) {
      this.addError(`${path}.requests_per_second`, 'Must be greater than 0');
    }
    
    if (typeof rateLimit.burst === 'number' && rateLimit.burst <= 0) {
      this.addError(`${path}.burst`, 'Must be greater than 0');
    }
  }
  
  private validateRetry(retry: any, path: string): void {
    this.validateRequiredField(retry, 'max_attempts', 'number', path);
    this.validateRequiredField(retry, 'backoff_ms', 'number', path);
    this.validateRequiredField(retry, 'exponential', 'boolean', path);
    this.validateRequiredField(retry, 'jitter', 'boolean', path);
    
    if (typeof retry.max_attempts === 'number' && retry.max_attempts <= 0) {
      this.addError(`${path}.max_attempts`, 'Must be greater than 0');
    }
    
    if (typeof retry.backoff_ms === 'number' && retry.backoff_ms < 0) {
      this.addError(`${path}.backoff_ms`, 'Must be non-negative');
    }
  }
  
  private validateS3(s3: any, path: string): void {
    this.validateRequiredField(s3, 'bucket', 'string', path);
    this.validateRequiredField(s3, 'prefix', 'string', path);
    this.validateRequiredField(s3, 'region', 'string', path);
  }
  
  private validateBrowser(browser: any, path: string): void {
    this.validateRequiredField(browser, 'headless', 'boolean', path);
    
    if (browser.viewport) {
      this.validateRequiredField(browser.viewport, 'width', 'number', `${path}.viewport`);
      this.validateRequiredField(browser.viewport, 'height', 'number', `${path}.viewport`);
      this.validateRequiredField(browser.viewport, 'device_scale_factor', 'number', `${path}.viewport`);
    }
    
    if (browser.args && !Array.isArray(browser.args)) {
      this.addError(`${path}.args`, 'Must be an array');
    }
  }
  
  private validateLogging(logging: any, path: string): void {
    this.validateRequiredField(logging, 'level', 'string', path);
    
    if (logging.level && !['debug', 'info', 'warn', 'error'].includes(logging.level)) {
      this.addError(`${path}.level`, 'Must be one of: debug, info, warn, error');
    }
    
    this.validateRequiredField(logging, 'structured', 'boolean', path);
    this.validateRequiredField(logging, 'include_screenshots', 'boolean', path);
  }
  
  private validateEffects(effects: any[]): void {
    const effectIds = new Set<string>();
    const dependencies = new Map<string, string[]>();
    
    effects.forEach((effect, index) => {
      const path = `effects[${index}]`;
      
      // Validate effect structure
      if (!this.isObject(effect)) {
        this.addError(path, 'Effect must be an object');
        return;
      }
      
      // Validate effect type
      const effectType = (effect as any).type;
      if (!effectType) {
        this.addError(`${path}.type`, 'Effect type is required');
        return;
      }
      
      // Get effect definition based on type
      const effectDef = this.getEffectDefinition(effect as any);
      if (!effectDef) {
        this.addError(path, `No definition found for effect type: ${effectType}`);
        return;
      }
      
      // Validate effect has required fields
      if (!effectDef.id) {
        this.addError(`${path}.${effectType}.id`, 'Effect ID is required');
      } else {
        // Check for duplicate IDs
        if (effectIds.has(effectDef.id)) {
          this.addError(`${path}.${effectType}.id`, `Duplicate effect ID: ${effectDef.id}`);
        } else {
          effectIds.add(effectDef.id);
        }
      }
      
      // Validate parameters exist
      if (!effectDef.params) {
        this.addError(`${path}.${effectType}.params`, 'Effect parameters are required');
      } else {
        // Validate specific parameters based on effect type
        this.validateEffectParams(effectType, effectDef.params, `${path}.${effectType}.params`);
      }
      
      // Store dependencies for later validation
      if (effectDef.dependencies && Array.isArray(effectDef.dependencies)) {
        dependencies.set(effectDef.id, effectDef.dependencies);
      }
    });
    
    // Validate all dependencies exist
    dependencies.forEach((deps, effectId) => {
      deps.forEach(dep => {
        if (!effectIds.has(dep)) {
          this.addError('effects', `Effect "${effectId}" depends on non-existent effect "${dep}"`);
        }
      });
    });
    
    // Check for circular dependencies
    this.validateNoCycles(dependencies);
  }
  
  private validateEffectParams(type: string, params: any, path: string): void {
    switch (type) {
      case 'navigate':
        this.validateRequiredField(params, 'url', 'string', path);
        break;
        
      case 'click':
        this.validateRequiredField(params, 'selector', 'string', path);
        break;
        
      case 'type':
        if (!Array.isArray(params.fields)) {
          this.addError(`${path}.fields`, 'Fields must be an array');
        } else {
          params.fields.forEach((field: any, i: number) => {
            this.validateRequiredField(field, 'selector', 'string', `${path}.fields[${i}]`);
            this.validateRequiredField(field, 'value', 'string', `${path}.fields[${i}]`);
          });
        }
        break;
        
      case 'extract_all':
        this.validateRequiredField(params, 'selector', 'string', path);
        this.validateRequiredField(params, 'fields', 'object', path);
        break;
        
      case 'validate':
        this.validateRequiredField(params, 'schema', 'string', path);
        if (!Array.isArray(params.rules)) {
          this.addError(`${path}.rules`, 'Rules must be an array');
        }
        break;
        
      case 'serialize_json':
        this.validateRequiredField(params, 'schema', 'string', path);
        break;
        
      case 'upload_s3':
        this.validateRequiredField(params, 'bucket', 'string', path);
        this.validateRequiredField(params, 'key_template', 'string', path);
        break;
    }
  }
  
  private validateSchemas(schemas: any): void {
    if (!this.isObject(schemas)) {
      this.addError('schemas', 'Schemas must be an object');
      return;
    }
    
    Object.entries(schemas).forEach(([name, schema]) => {
      const path = `schemas.${name}`;
      
      if (!this.isObject(schema)) {
        this.addError(path, 'Schema must be an object');
        return;
      }
      
      const s = schema as any;
      this.validateRequiredField(s, 'name', 'string', path);
      
      if (!Array.isArray(s.fields)) {
        this.addError(`${path}.fields`, 'Schema fields must be an array');
      }
    });
  }
  
  private validateNoCycles(dependencies: Map<string, string[]>): void {
    const visited = new Set<string>();
    const recursionStack = new Set<string>();
    
    const hasCycle = (node: string): boolean => {
      visited.add(node);
      recursionStack.add(node);
      
      const deps = dependencies.get(node) || [];
      for (const dep of deps) {
        if (!visited.has(dep)) {
          if (hasCycle(dep)) return true;
        } else if (recursionStack.has(dep)) {
          return true;
        }
      }
      
      recursionStack.delete(node);
      return false;
    };
    
    for (const node of dependencies.keys()) {
      if (!visited.has(node)) {
        if (hasCycle(node)) {
          this.addError('effects', 'Circular dependency detected in effect dependencies');
          break;
        }
      }
    }
  }
  
  private getEffectDefinition(effect: any): any {
    // Handle both JSON formats:
    // 1. Direct format (from JSON): { type, id, dependencies, params }
    // 2. Nested format (from TypeScript): { type, [type_key]: { id, dependencies, params } }
    
    // If effect has id directly, it's the direct format
    if (effect.id) {
      return {
        id: effect.id,
        dependencies: effect.dependencies,
        params: effect.params
      };
    }
    
    // Otherwise, try the nested format
    const typeMap: Record<string, string> = {
      'navigate': 'navigate',
      'click': 'click',
      'type': 'type_effect',
      'wait_for_selector': 'wait_for_selector',
      'extract_all': 'extract_all',
      'extract_text': 'extract_text',
      'screenshot': 'screenshot',
      'api_request': 'api_request',
      'extract_auth_token': 'extract_auth_token',
      'delay': 'delay',
      'conditional': 'conditional',
      'retry': 'retry',
      'validate': 'validate',
      'serialize_json': 'serialize_json',
      'transform': 'transform',
      'upload_s3': 'upload_s3',
      'save_local': 'save_local'
    };
    
    const key = typeMap[effect.type];
    return key ? effect[key] : null;
  }
  
  private validateRequiredField(
    obj: any,
    field: string,
    type: string,
    parentPath?: string
  ): boolean {
    const path = parentPath ? `${parentPath}.${field}` : field;
    
    if (!(field in obj)) {
      this.addError(path, `${field} is required`);
      return false;
    }
    
    const value = obj[field];
    const actualType = Array.isArray(value) ? 'array' : typeof value;
    
    if (actualType !== type) {
      this.addError(path, `${field} must be of type ${type}, got ${actualType}`);
      return false;
    }
    
    return true;
  }
  
  private isObject(value: unknown): value is object {
    return value !== null && typeof value === 'object' && !Array.isArray(value);
  }
  
  private addError(path: string, message: string, value?: any): void {
    this.errors.push({ path, message, value });
  }
  
  private getResult(): ValidationResult {
    return {
      valid: this.errors.length === 0,
      errors: this.errors
    };
  }
}