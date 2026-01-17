// Control Flow Effects

import { BaseEffect, EffectError, ExecutionContext } from './index';
import { DelayParams, ConditionalParams } from '../types';

export class DelayEffect extends BaseEffect<DelayParams, void, void> {
  async execute(_: void, _context: ExecutionContext): Promise<void> {
    let delayMs = this.params.milliseconds;
    
    // Add jitter if specified
    if (this.params.jitter) {
      const jitterAmount = Math.random() * this.params.jitter;
      delayMs += Math.round(jitterAmount - this.params.jitter / 2);
    }
    
    this.log('info', `Delaying for ${delayMs}ms`);
    
    await new Promise(resolve => setTimeout(resolve, delayMs));
    
    this.log('debug', `Delay completed`);
  }
  
  override validate(): void {
    super.validate();
    
    if (this.params.milliseconds <= 0) {
      throw new Error('Delay milliseconds must be greater than 0');
    }
  }
}

export class ConditionalEffect extends BaseEffect<ConditionalParams, any, { branch: 'then' | 'else'; effects: string[] }> {
  async execute(input: any, context: ExecutionContext): Promise<{ branch: 'then' | 'else'; effects: string[] }> {
    this.log('info', `Evaluating condition`);
    
    try {
      const conditionMet = await this.evaluateCondition(input, context);
      
      const branch = conditionMet ? 'then' : 'else';
      const effects = conditionMet ? this.params.then_effects : this.params.else_effects;
      
      this.log('info', `Condition evaluated to ${conditionMet}, taking ${branch} branch with ${effects.length} effects`);
      
      // Store the decision in context
      context.data.set(`${this.id}_branch`, branch);
      context.data.set(`${this.id}_effects`, effects);
      
      return { branch, effects };
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        'Failed to evaluate condition',
        error as Error
      );
    }
  }
  
  private async evaluateCondition(input: any, context: ExecutionContext): Promise<boolean> {
    const { condition } = this.params;
    
    let actualValue: any;
    
    switch (condition.expression_type) {
      case 'context_value':
        actualValue = context.data.get(condition.field);
        break;
        
      case 'effect_result':
        actualValue = this.getNestedValue(input, condition.field);
        break;
        
      case 'selector_exists':
        try {
          const element = await context.page.$(condition.field);
          actualValue = element !== null;
        } catch {
          actualValue = false;
        }
        break;
        
      default:
        throw new Error(`Unknown condition type: ${condition.expression_type}`);
    }
    
    return this.compareValues(actualValue, condition.operator, condition.value);
  }
  
  private compareValues(actual: any, operator: string, expected: any): boolean {
    switch (operator) {
      case 'equals':
        return actual === expected;
        
      case 'not_equals':
        return actual !== expected;
        
      case 'greater_than':
        return Number(actual) > Number(expected);
        
      case 'less_than':
        return Number(actual) < Number(expected);
        
      case 'contains':
        if (typeof actual === 'string' && typeof expected === 'string') {
          return actual.includes(expected);
        }
        if (Array.isArray(actual)) {
          return actual.includes(expected);
        }
        return false;
        
      case 'exists':
        return actual !== null && actual !== undefined;
        
      default:
        throw new Error(`Unknown operator: ${operator}`);
    }
  }
  
  private getNestedValue(obj: any, path: string): any {
    const parts = path.split('.');
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
  
  override validate(): void {
    super.validate();
    
    if (this.params.then_effects.length === 0 && this.params.else_effects.length === 0) {
      throw new Error('At least one of then_effects or else_effects must have effects');
    }
  }
}

// Retry wrapper effect
export class RetryEffect extends BaseEffect<{ effect_id: string; max_attempts: number; backoff_ms: number }, any, any> {
  async execute(input: any, _context: ExecutionContext): Promise<any> {
    this.log('info', `Retrying effect ${this.params.effect_id} up to ${this.params.max_attempts} times`);
    
    let lastError: Error | undefined;
    
    for (let attempt = 1; attempt <= this.params.max_attempts; attempt++) {
      try {
        this.log('info', `Attempt ${attempt}/${this.params.max_attempts}`);
        
        // This would need to be implemented differently in practice
        // as we'd need to execute the actual effect
        
        return input; // Placeholder
      } catch (error) {
        lastError = error as Error;
        this.log('warn', `Attempt ${attempt} failed: ${lastError.message}`);
        
        if (attempt < this.params.max_attempts) {
          const backoffMs = this.params.backoff_ms * Math.pow(2, attempt - 1);
          this.log('info', `Backing off for ${backoffMs}ms`);
          await new Promise(resolve => setTimeout(resolve, backoffMs));
        }
      }
    }
    
    throw new EffectError(
      this.id,
      this.type,
      `All ${this.params.max_attempts} attempts failed`,
      lastError
    );
  }
  
  override validate(): void {
    super.validate();
    
    if (!this.params.effect_id) {
      throw new Error('Effect ID to retry is required');
    }
    
    if (this.params.max_attempts <= 0) {
      throw new Error('Max attempts must be greater than 0');
    }
  }
}