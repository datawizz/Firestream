// Effect Factory
// Creates effect instances from JSON definitions

import { Effect } from '../effects';
import {
  NavigateEffect,
  ClickEffect,
  TypeEffect,
  WaitForSelectorEffect,
  ExtractAllEffect,
  ExtractTextEffect,
  ScreenshotEffect,
  ApiRequestEffect,
  ExtractAuthTokenEffect,
  ValidateEffect,
  SerializeJsonEffect,
  UploadS3Effect,
  DelayEffect,
  ConditionalEffect
} from '../effects';

import {
  EffectDefinition,
  WorkflowDefinition,
  WorkflowConfig
} from '../types';

import { WorkflowExecutor } from './executor';
import { RateLimiter } from './rate-limiter';
import { RetryManager } from './retry';
import { EffectRegistry } from './registry';
import { WorkflowValidator } from './validator';
import { logger } from '../utils/logger';

export class EffectFactory {
  private registry: EffectRegistry = new EffectRegistry();
  
  createEffect(definition: EffectDefinition): Effect<any, any, any> {
    switch (definition.type) {
      case 'navigate':
        if (!definition.navigate) {
          throw new Error('Navigate definition is missing');
        }
        return new NavigateEffect(
          definition.navigate.id,
          'navigate',
          definition.navigate.params,
          definition.navigate.dependencies || []
        );
        
      case 'click':
        if (!definition.click) {
          throw new Error('Click definition is missing');
        }
        return new ClickEffect(
          definition.click.id,
          'click',
          definition.click.params,
          definition.click.dependencies || []
        );
        
      case 'type':
        if (!definition.type_effect) {
          throw new Error('Type effect definition is missing');
        }
        return new TypeEffect(
          definition.type_effect.id,
          'type',
          definition.type_effect.params,
          definition.type_effect.dependencies || []
        );
        
      case 'wait_for_selector':
        if (!definition.wait_for_selector) {
          throw new Error('Wait for selector definition is missing');
        }
        return new WaitForSelectorEffect(
          definition.wait_for_selector.id,
          'wait_for_selector',
          definition.wait_for_selector.params,
          definition.wait_for_selector.dependencies || []
        );
        
      case 'extract_all':
        if (!definition.extract_all) {
          throw new Error('Extract all definition is missing');
        }
        return new ExtractAllEffect(
          definition.extract_all.id,
          'extract_all',
          definition.extract_all.params,
          definition.extract_all.dependencies || []
        );
        
      case 'extract_text':
        if (!definition.extract_text) {
          throw new Error('Extract text definition is missing');
        }
        return new ExtractTextEffect(
          definition.extract_text.id,
          'extract_text',
          definition.extract_text.params,
          definition.extract_text.dependencies || []
        );
        
      case 'screenshot':
        if (!definition.screenshot) {
          throw new Error('Screenshot definition is missing');
        }
        return new ScreenshotEffect(
          definition.screenshot.id,
          'screenshot',
          definition.screenshot.params,
          definition.screenshot.dependencies || []
        );
        
      case 'api_request':
        if (!definition.api_request) {
          throw new Error('API request definition is missing');
        }
        return new ApiRequestEffect(
          definition.api_request.id,
          'api_request',
          definition.api_request.params,
          definition.api_request.dependencies || []
        );
        
      case 'validate':
        if (!definition.validate) {
          throw new Error('Validate definition is missing');
        }
        return new ValidateEffect(
          definition.validate.id,
          'validate',
          definition.validate.params,
          definition.validate.dependencies || []
        );
        
      case 'serialize_json':
        if (!definition.serialize_json) {
          throw new Error('Serialize JSON definition is missing');
        }
        return new SerializeJsonEffect(
          definition.serialize_json.id,
          'serialize_json',
          definition.serialize_json.params,
          definition.serialize_json.dependencies || []
        );
        
      case 'upload_s3':
        if (!definition.upload_s3) {
          throw new Error('Upload S3 definition is missing');
        }
        return new UploadS3Effect(
          definition.upload_s3.id,
          'upload_s3',
          definition.upload_s3.params,
          definition.upload_s3.dependencies || []
        );
        
      case 'delay':
        if (!definition.delay) {
          throw new Error('Delay definition is missing');
        }
        return new DelayEffect(
          definition.delay.id,
          'delay',
          definition.delay.params,
          definition.delay.dependencies || []
        );
        
      case 'conditional':
        if (!definition.conditional) {
          throw new Error('Conditional definition is missing');
        }
        return new ConditionalEffect(
          definition.conditional.id,
          'conditional',
          definition.conditional.params,
          definition.conditional.dependencies || []
        );
        
      default:
        throw new Error(`Unknown effect type: ${(definition as any).type}`);
    }
  }
  
  createAllEffects(workflow: WorkflowDefinition): Map<string, Effect<any, any, any>> {
    this.registry.clear();
    
    for (const definition of workflow.effects) {
      const effect = this.createEffect(definition);
      this.registry.register(effect);
    }
    
    // Validate all dependencies exist
    this.registry.validateDependencies();
    
    logger.info(`Created ${this.registry.getAll().size} effects for workflow ${workflow.workflow_id}`);
    return this.registry.getAll();
  }
  
  getEffect(id: string): Effect<any, any, any> | undefined {
    return this.registry.get(id);
  }
  
  getRegistry(): EffectRegistry {
    return this.registry;
  }
}

// Main factory function to create executor with all dependencies
export function createExecutor(workflow: WorkflowDefinition | unknown): WorkflowExecutor {
  // Validate workflow structure
  const validator = new WorkflowValidator();
  const validationResult = validator.validate(workflow);
  
  if (!validationResult.valid) {
    const errorMessages = validationResult.errors
      .map(e => `${e.path}: ${e.message}`)
      .join('\n');
    throw new Error(`Invalid workflow definition:\n${errorMessages}`);
  }
  
  // Type assertion is safe after validation
  const validWorkflow = workflow as WorkflowDefinition;
  
  // Create rate limiter
  const rateLimiter = new RateLimiter({
    requestsPerSecond: validWorkflow.config.rate_limit.requests_per_second,
    burst: validWorkflow.config.rate_limit.burst,
    perDomain: validWorkflow.config.rate_limit.per_domain
  });
  
  // Create retry manager
  const retryManager = new RetryManager({
    maxAttempts: validWorkflow.config.retry.max_attempts,
    backoffMs: validWorkflow.config.retry.backoff_ms,
    exponential: validWorkflow.config.retry.exponential,
    jitter: validWorkflow.config.retry.jitter
  });
  
  // Create effect factory and build all effects
  const factory = new EffectFactory();
  const effectMap = factory.createAllEffects(validWorkflow);
  
  // Validate effect references
  validateEffectReferences(validWorkflow);
  
  // Create executor with actual effect instances
  return new WorkflowExecutor(
    validWorkflow,
    effectMap,
    rateLimiter,
    retryManager
  );
}

// Helper to create auth extraction effect if needed
export function createAuthExtractionEffect(
  workflowConfig: WorkflowConfig
): ExtractAuthTokenEffect | undefined {
  if (!workflowConfig.auth?.token_extraction) {
    return undefined;
  }
  
  const { storage_type, token_keys } = workflowConfig.auth.token_extraction;
  
  return new ExtractAuthTokenEffect(
    'extract_auth',
    'extract_auth_token',
    { storage_type, token_keys },
    []
  );
}

// Validate effect dependencies exist
export function validateEffectReferences(workflow: WorkflowDefinition): void {
  const effectIds = new Set<string>();
  
  // First collect all effect IDs
  for (const definition of workflow.effects) {
    const effectDef = getEffectDefinitionData(definition);
    if (effectDef) {
      effectIds.add(effectDef.id);
    }
  }
  
  // Then validate dependencies
  for (const definition of workflow.effects) {
    const effectDef = getEffectDefinitionData(definition);
    if (effectDef && effectDef.dependencies) {
      for (const dep of effectDef.dependencies) {
        if (!effectIds.has(dep)) {
          throw new Error(
            `Effect "${effectDef.id}" depends on non-existent effect "${dep}"`
          );
        }
      }
    }
  }
  
  logger.debug('Effect references validated successfully');
}

// Helper to get effect definition data
function getEffectDefinitionData(definition: EffectDefinition): { id: string; dependencies?: string[] } | null {
  switch (definition.type) {
    case 'navigate': return definition.navigate || null;
    case 'click': return definition.click || null;
    case 'type': return definition.type_effect || null;
    case 'wait_for_selector': return definition.wait_for_selector || null;
    case 'extract_all': return definition.extract_all || null;
    case 'extract_text': return definition.extract_text || null;
    case 'screenshot': return definition.screenshot || null;
    case 'api_request': return definition.api_request || null;
    case 'extract_auth_token': return definition.extract_auth_token || null;
    case 'delay': return definition.delay || null;
    case 'conditional': return definition.conditional || null;
    case 'retry': return definition.retry || null;
    case 'validate': return definition.validate || null;
    case 'serialize_json': return definition.serialize_json || null;
    case 'transform': return definition.transform || null;
    case 'upload_s3': return definition.upload_s3 || null;
    case 'save_local': return definition.save_local || null;
    default: return null;
  }
}