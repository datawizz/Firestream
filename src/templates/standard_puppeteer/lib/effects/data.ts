// Data Processing Effects

import { BaseEffect, EffectError, ExecutionContext } from './index';
import { ValidateParams, SerializeJsonParams } from '../types';

export class ValidateEffect extends BaseEffect<ValidateParams, any[], any[]> {
  async execute(input: any[], context: ExecutionContext): Promise<any[]> {
    this.log('info', `Validating ${input.length} items against schema ${this.params.schema}`);
    
    try {
      const errors: Array<{ index: number; field: string; message: string }> = [];
      const validItems: any[] = [];
      
      // Apply validation rules
      for (let i = 0; i < input.length; i++) {
        const item = input[i];
        const itemErrors = this.validateItem(item, i);
        
        if (itemErrors.length > 0) {
          errors.push(...itemErrors);
          if (this.params.fail_on_error) {
            break;
          }
        } else {
          validItems.push(item);
        }
      }
      
      if (errors.length > 0) {
        this.log('warn', `Validation found ${errors.length} errors`, errors);
        
        if (this.params.fail_on_error) {
          throw new EffectError(
            this.id,
            this.type,
            `Validation failed with ${errors.length} errors`,
            new Error(JSON.stringify(errors))
          );
        }
      }
      
      this.log('info', `Validated ${validItems.length} items successfully`);
      
      // Store validation results
      context.data.set(`${this.id}_valid_count`, validItems.length);
      context.data.set(`${this.id}_error_count`, errors.length);
      context.data.set(`${this.id}_errors`, errors);
      
      return validItems;
    } catch (error) {
      if (error instanceof EffectError) {
        throw error;
      }
      
      throw new EffectError(
        this.id,
        this.type,
        'Validation process failed',
        error as Error
      );
    }
  }
  
  private validateItem(item: any, index: number): Array<{ index: number; field: string; message: string }> {
    const errors: Array<{ index: number; field: string; message: string }> = [];
    
    for (const rule of this.params.rules) {
      const value = this.getFieldValue(item, rule.field);
      
      switch (rule.rule_type) {
        case 'required':
          if (value === null || value === undefined || value === '') {
            errors.push({
              index,
              field: rule.field,
              message: rule.message || `${rule.field} is required`
            });
          }
          break;
          
        case 'min':
          if (typeof value === 'number' && rule.value !== undefined && typeof rule.value === 'number' && value < rule.value) {
            errors.push({
              index,
              field: rule.field,
              message: rule.message || `${rule.field} must be at least ${rule.value}`
            });
          }
          break;
          
        case 'max':
          if (typeof value === 'number' && rule.value !== undefined && typeof rule.value === 'number' && value > rule.value) {
            errors.push({
              index,
              field: rule.field,
              message: rule.message || `${rule.field} must be at most ${rule.value}`
            });
          }
          break;
          
        case 'pattern':
          if (typeof value === 'string' && rule.value) {
            const pattern = new RegExp(rule.value as string);
            if (!pattern.test(value)) {
              errors.push({
                index,
                field: rule.field,
                message: rule.message || `${rule.field} does not match pattern ${rule.value}`
              });
            }
          }
          break;
          
        case 'custom':
          // Custom validation would be implemented based on specific requirements
          break;
      }
    }
    
    return errors;
  }
  
  private getFieldValue(obj: any, fieldPath: string): any {
    const parts = fieldPath.split('.');
    let value = obj;
    
    for (const part of parts) {
      if (value && typeof value === 'object') {
        value = value[part];
      } else {
        return undefined;
      }
    }
    
    return value;
  }
}

export class SerializeJsonEffect extends BaseEffect<SerializeJsonParams, any[], string> {
  async execute(input: any[], context: ExecutionContext): Promise<string> {
    this.log('info', `Serializing ${input.length} items to JSON`);
    
    try {
      const data: any = {
        workflow_id: context.workflowId,
        execution_id: context.executionId,
        timestamp: new Date().toISOString(),
        item_count: input.length,
        items: input
      };
      
      if (this.params.add_metadata) {
        data.metadata = {
          schema: this.params.schema,
          example_shop_version: '1.0.0',
          extraction_date: new Date().toISOString(),
          environment: process.env['NODE_ENV'] || 'production'
        };
      }
      
      const json = this.params.pretty 
        ? JSON.stringify(data, null, 2)
        : JSON.stringify(data);
      
      this.log('info', `Serialized to JSON: ${json.length} bytes`);
      
      // Store serialized data for next effect (e.g., upload)
      context.data.set(`${this.id}_json`, json);
      context.data.set(`${this.id}_size`, json.length);
      
      return json;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to serialize data to JSON',
        error as Error
      );
    }
  }
}

// Transform effect for data manipulation
export class TransformEffect extends BaseEffect<{ transformations: Array<{ field: string; operation: string }> }, any[], any[]> {
  async execute(input: any[], _context: ExecutionContext): Promise<any[]> {
    this.log('info', `Applying ${this.params.transformations.length} transformations to ${input.length} items`);
    
    try {
      const transformed = input.map(item => {
        const newItem = { ...item };
        
        for (const transform of this.params.transformations) {
          const value = this.getFieldValue(newItem, transform.field);
          const transformed = this.applyTransformation(value, transform.operation);
          this.setFieldValue(newItem, transform.field, transformed);
        }
        
        return newItem;
      });
      
      this.log('info', 'Transformations applied successfully');
      return transformed;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to transform data',
        error as Error
      );
    }
  }
  
  private getFieldValue(obj: any, fieldPath: string): any {
    const parts = fieldPath.split('.');
    let value = obj;
    
    for (const part of parts) {
      if (value && typeof value === 'object') {
        value = value[part];
      } else {
        return undefined;
      }
    }
    
    return value;
  }
  
  private setFieldValue(obj: any, fieldPath: string, value: any): void {
    const parts = fieldPath.split('.');
    let current = obj;
    
    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      if (part !== undefined && (!current[part] || typeof current[part] !== 'object')) {
        current[part] = {};
      }
      if (part !== undefined) {
        current = current[part];
      }
    }
    
    const lastPart = parts[parts.length - 1];
    if (lastPart !== undefined) {
      current[lastPart] = value;
    }
  }
  
  private applyTransformation(value: any, operation: string): any {
    if (value === null || value === undefined) return value;
    
    switch (operation) {
      case 'lowercase':
        return typeof value === 'string' ? value.toLowerCase() : value;
      case 'uppercase':
        return typeof value === 'string' ? value.toUpperCase() : value;
      case 'trim':
        return typeof value === 'string' ? value.trim() : value;
      case 'parse_number':
        return typeof value === 'string' ? parseFloat(value.replace(/[^0-9.-]/g, '')) : value;
      case 'parse_date':
        return new Date(value).toISOString();
      default:
        return value;
    }
  }
}