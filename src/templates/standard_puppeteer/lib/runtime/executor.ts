// Workflow Executor
// Handles the execution of effects in dependency order with rate limiting

import { ExecutionContext, Effect } from '../effects';
import { WorkflowDefinition, WorkflowResult, WorkflowError } from '../types';
import { RateLimiter } from './rate-limiter';
import { RetryManager } from './retry';
import { logger } from '../utils/logger';
import { Browser, Page } from 'puppeteer';
import puppeteer from 'puppeteer';

export class WorkflowExecutor {
  private executionId: string;
  private startTime: Date;
  private context!: ExecutionContext; // Will be initialized in execute()
  private effectResults: Map<string, any> = new Map();
  private effectErrors: Map<string, Error> = new Map();
  
  constructor(
    private workflow: WorkflowDefinition,
    private effects: Map<string, Effect<any, any, any>>,
    private rateLimiter: RateLimiter,
    private retryManager: RetryManager,
    private browser?: Browser
  ) {
    this.executionId = this.generateExecutionId();
    this.startTime = new Date();
  }
  
  async execute(): Promise<WorkflowResult> {
    logger.info(`Starting workflow execution: ${this.workflow.workflow_id}`, {
      executionId: this.executionId,
      effectCount: this.workflow.effects.length
    });
    
    let browser: Browser | undefined;
    let page: Page | undefined;
    
    try {
      // Initialize browser and page
      const executablePath = this.workflow.config.browser?.executable_path || process.env['PUPPETEER_EXECUTABLE_PATH'];
      browser = this.browser || await puppeteer.launch({
        headless: this.workflow.config.browser?.headless ?? true,
        args: this.workflow.config.browser?.args || ['--no-sandbox', '--disable-setuid-sandbox'],
        ...(executablePath && { executablePath })
      });
      
      page = await browser.newPage();
      
      // Set viewport
      if (this.workflow.config.browser?.viewport) {
        await page.setViewport({
          width: this.workflow.config.browser.viewport.width,
          height: this.workflow.config.browser.viewport.height,
          deviceScaleFactor: this.workflow.config.browser.viewport.device_scale_factor
        });
      }
      
      // Set user agent
      if (this.workflow.config.browser?.user_agent) {
        await page.setUserAgent(this.workflow.config.browser.user_agent);
      }
      
      // Initialize execution context
      this.context = {
        browser,
        page,
        data: new Map(),
        logger,
        workflowId: this.workflow.workflow_id,
        executionId: this.executionId
      };
      
      // Execute effects in dependency order
      const executionOrder = this.getExecutionOrder();
      
      for (const effectId of executionOrder) {
        await this.executeEffect(effectId);
      }
      
      // Get final results
      const errors: WorkflowError[] = Array.from(this.effectErrors.entries()).map(([effectId, error]) => {
        const workflowError: WorkflowError = {
          effect_id: effectId,
          error_type: error.name,
          message: error.message,
          timestamp: new Date(),
          retry_count: 0
        };
        if (error.stack) {
          workflowError.stack = error.stack;
        }
        return workflowError;
      });
      
      const result: WorkflowResult = {
        workflow_id: this.workflow.workflow_id,
        execution_id: this.executionId,
        status: this.effectErrors.size > 0 ? 'completed' : 'completed', // Both partial and full success map to 'completed'
        start_time: this.startTime,
        end_time: new Date(),
        duration_ms: Date.now() - this.startTime.getTime(),
        data: Object.fromEntries(this.effectResults),
        errors,
        screenshots: this.getScreenshots(),
        logs: []
      };
      
      logger.info(`Workflow completed: ${this.workflow.workflow_id}`, {
        executionId: this.executionId,
        status: result.status,
        duration: result.duration_ms,
        dataCount: Object.keys(result.data).length,
        errors: result.errors.length
      });
      
      return result;
      
    } catch (error) {
      logger.error(`Workflow failed: ${this.workflow.workflow_id}`, {
        executionId: this.executionId,
        error: error instanceof Error ? error.message : 'Unknown error',
        stack: error instanceof Error ? error.stack : undefined
      });
      
      return {
        workflow_id: this.workflow.workflow_id,
        execution_id: this.executionId,
        status: 'failed',
        start_time: this.startTime,
        end_time: new Date(),
        duration_ms: Date.now() - this.startTime.getTime(),
        data: {},
        errors: [(() => {
          const workflowError: WorkflowError = {
            effect_id: 'workflow',
            error_type: error instanceof Error ? error.name : 'Error',
            message: error instanceof Error ? error.message : 'Unknown error',
            timestamp: new Date(),
            retry_count: 0
          };
          if (error instanceof Error && error.stack) {
            workflowError.stack = error.stack;
          }
          return workflowError;
        })()],
        screenshots: [],
        logs: []
      };
      
    } finally {
      // Cleanup
      if (page && !this.browser) {
        await page.close();
      }
      if (browser && !this.browser) {
        await browser.close();
      }
    }
  }
  
  private async executeEffect(effectId: string): Promise<void> {
    const effect = this.effects.get(effectId);
    if (!effect) {
      throw new Error(`Effect not found: ${effectId}`);
    }
    
    logger.info(`Executing effect: ${effectId}`, {
      type: effect.type,
      dependencies: effect.dependencies
    });
    
    try {
      // Apply rate limiting
      await this.rateLimiter.acquire();
      
      // Get input from dependencies
      const input = this.getEffectInput(effect);
      
      // Execute with retry
      const result = await this.retryManager.execute(
        async () => effect.execute(input, this.context),
        {
          maxAttempts: this.workflow.config.retry.max_attempts,
          backoffMs: this.workflow.config.retry.backoff_ms,
          exponential: this.workflow.config.retry.exponential,
          jitter: this.workflow.config.retry.jitter
        }
      );
      
      // Store result
      this.effectResults.set(effectId, result);
      
      logger.info(`Effect completed: ${effectId}`);
      
    } catch (error) {
      logger.error(`Effect failed: ${effectId}`, {
        error: error instanceof Error ? error.message : 'Unknown error',
        stack: error instanceof Error ? error.stack : undefined
      });
      
      this.effectErrors.set(effectId, error as Error);
      
      // Determine if we should continue
      if (this.isCriticalEffect(effect)) {
        throw error;
      }
    }
  }
  
  private getExecutionOrder(): string[] {
    // Topological sort to determine execution order
    const graph = new Map<string, string[]>();
    const inDegree = new Map<string, number>();
    
    // Build graph
    for (const definition of this.workflow.effects) {
      const effectDef = this.getEffectDefinitionData(definition);
      if (!effectDef) continue;
      
      const id = effectDef.id;
      const deps = effectDef.dependencies || [];
      graph.set(id, deps);
      inDegree.set(id, deps.length);
    }
    
    // Find effects with no dependencies
    const queue: string[] = [];
    for (const [id, degree] of inDegree.entries()) {
      if (degree === 0) {
        queue.push(id);
      }
    }
    
    // Process queue
    const order: string[] = [];
    while (queue.length > 0) {
      const current = queue.shift()!;
      order.push(current);
      
      // Update dependent effects
      for (const [id, deps] of graph.entries()) {
        if (deps.includes(current)) {
          const newDegree = inDegree.get(id)! - 1;
          inDegree.set(id, newDegree);
          
          if (newDegree === 0) {
            queue.push(id);
          }
        }
      }
    }
    
    if (order.length !== this.workflow.effects.length) {
      throw new Error('Circular dependency detected in workflow');
    }
    
    return order;
  }
  
  private getEffectInput(effect: Effect<any, any, any>): any {
    const dependencies = effect.dependencies;
    
    if (dependencies.length === 0) {
      return undefined;
    }
    
    if (dependencies.length === 1) {
      const dep = dependencies[0];
      return dep ? this.effectResults.get(dep) : undefined;
    }
    
    // Multiple dependencies - return as object
    const inputs: Record<string, any> = {};
    for (const dep of dependencies) {
      inputs[dep] = this.effectResults.get(dep);
    }
    return inputs;
  }
  
  private isCriticalEffect(effect: Effect<any, any, any>): boolean {
    // Effects that should stop execution if they fail
    const criticalTypes = ['navigate', 'wait_for_selector', 'validate', 'serialize_json'];
    return criticalTypes.includes(effect.type);
  }
  
  private getScreenshots(): string[] {
    const screenshots: string[] = [];
    
    for (const [key, value] of this.context.data.entries()) {
      if (key.endsWith('_screenshot_path') && typeof value === 'string') {
        screenshots.push(value);
      }
    }
    
    return screenshots;
  }
  
  
  private getEffectDefinitionData(definition: any): { id: string; dependencies?: string[] } | null {
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
  
  private generateExecutionId(): string {
    return `${this.workflow.workflow_id}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}