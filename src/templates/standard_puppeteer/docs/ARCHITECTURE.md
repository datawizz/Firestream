# E-commerce Scraper Architecture

## Table of Contents

1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [System Architecture](#system-architecture)
4. [Effects System](#effects-system)
5. [Workflow Engine](#workflow-engine)
6. [Runtime Context](#runtime-context)
7. [Data Flow](#data-flow)
8. [Error Handling](#error-handling)
9. [Performance Considerations](#performance-considerations)
10. [Extension Points](#extension-points)

## Overview

The E-commerce Scraper employs an effects-based architecture that separates concerns and provides a modular, extensible system for web scraping. This architecture enables:

- **Declarative workflow definitions**
- **Reusable scraping components**
- **Dependency-based execution**
- **Built-in resilience patterns**
- **Type-safe implementations**

## Core Concepts

### Effects

Effects are the fundamental building blocks of the scraper. Each effect represents a single, atomic operation:

```typescript
interface Effect<TParams, TInput, TOutput> {
  id: string;
  type: string;
  params: TParams;
  dependencies: string[];
  execute(input: TInput, context: ExecutionContext): Promise<TOutput>;
}
```

### Workflows

Workflows are collections of effects with defined dependencies:

```typescript
interface WorkflowDefinition {
  workflow_id: string;
  effects: EffectDefinition[];
  config: WorkflowConfig;
  schemas: SchemaDefinitions;
}
```

### Execution Context

The execution context provides shared resources and state:

```typescript
interface ExecutionContext {
  browser: Browser;
  page: Page;
  data: Map<string, any>;
  logger: Logger;
  workflowId: string;
  executionId: string;
}
```

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Entry Point                          │
│                      (src/index.ts)                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                    Workflow Loader                           │
│                 (src/workflow.ts)                           │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                   Execution Engine                           │
│              (lib/runtime/executor.ts)                      │
├─────────────────────────────────────────────────────────────┤
│  • Dependency Resolution (Topological Sort)                 │
│  • Effect Orchestration                                      │
│  • Context Management                                        │
│  • Error Aggregation                                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┬─────────────────────┐
        │                           │                     │
┌───────▼──────────┐ ┌─────────────▼────────┐ ┌─────────▼────────┐
│  Rate Limiter    │ │   Retry Manager      │ │  Effect Factory  │
│(rate-limiter.ts) │ │    (retry.ts)        │ │  (factory.ts)    │
└──────────────────┘ └────────────────────────┘ └──────────────────┘
                                                          │
                                    ┌─────────────────────┴────────────┐
                                    │                                  │
                            ┌───────▼──────────┐            ┌─────────▼────────┐
                            │   DOM Effects    │            │   API Effects    │
                            │   (dom.ts)       │            │   (api.ts)       │
                            └──────────────────┘            └──────────────────┘
```

## Effects System

### Effect Categories

1. **DOM Manipulation Effects**
   - `NavigateEffect`: Page navigation
   - `ClickEffect`: Element clicking
   - `TypeEffect`: Form input
   - `WaitForSelectorEffect`: Element waiting
   - `ExtractAllEffect`: Bulk data extraction
   - `ExtractTextEffect`: Text extraction
   - `ScreenshotEffect`: Screenshot capture

2. **API Effects**
   - `ApiRequestEffect`: HTTP requests
   - `ExtractAuthTokenEffect`: Token extraction

3. **Control Flow Effects**
   - `DelayEffect`: Execution delays
   - `ConditionalEffect`: Conditional branching
   - `RetryEffect`: Effect retry logic

4. **Data Processing Effects**
   - `ValidateEffect`: Schema validation
   - `SerializeJsonEffect`: JSON serialization
   - `TransformEffect`: Data transformation

5. **Storage Effects**
   - `UploadS3Effect`: S3 uploads
   - `SaveLocalEffect`: Local file storage

### Effect Base Class

```typescript
export abstract class BaseEffect<TParams, TInput, TOutput> {
  constructor(
    protected id: string,
    protected type: string,
    protected params: TParams,
    protected dependencies: string[] = []
  ) {}

  abstract execute(input: TInput, context: ExecutionContext): Promise<TOutput>;

  protected log(level: string, message: string, meta?: any): void {
    logger.log(level, message, {
      effectId: this.id,
      effectType: this.type,
      ...meta
    });
  }
}
```

### Effect Lifecycle

1. **Initialization**: Effect instance created from definition
2. **Dependency Resolution**: Input data gathered from dependencies
3. **Rate Limiting**: Request throttling applied
4. **Execution**: Effect logic runs
5. **Retry Logic**: Automatic retry on failure
6. **Result Storage**: Output stored in context
7. **Error Handling**: Errors captured and logged

## Workflow Engine

### Dependency Resolution

The workflow engine uses topological sorting to determine execution order:

```typescript
private getExecutionOrder(): string[] {
  const graph = new Map<string, string[]>();
  const inDegree = new Map<string, number>();
  
  // Build dependency graph
  for (const effect of this.workflow.effects) {
    graph.set(effect.id, effect.dependencies);
    inDegree.set(effect.id, effect.dependencies.length);
  }
  
  // Topological sort
  const queue: string[] = [];
  const order: string[] = [];
  
  // Find nodes with no dependencies
  for (const [id, degree] of inDegree.entries()) {
    if (degree === 0) queue.push(id);
  }
  
  // Process queue
  while (queue.length > 0) {
    const current = queue.shift()!;
    order.push(current);
    
    // Update dependent nodes
    for (const [id, deps] of graph.entries()) {
      if (deps.includes(current)) {
        const newDegree = inDegree.get(id)! - 1;
        inDegree.set(id, newDegree);
        if (newDegree === 0) queue.push(id);
      }
    }
  }
  
  return order;
}
```

### Execution Flow

1. **Initialization**
   - Create browser instance
   - Set up page with viewport and user agent
   - Initialize execution context

2. **Effect Execution**
   - Resolve dependencies
   - Apply rate limiting
   - Execute effect with retry logic
   - Store results or errors

3. **Completion**
   - Aggregate results
   - Generate execution report
   - Clean up resources

## Runtime Context

### Context Components

```typescript
interface ExecutionContext {
  // Browser Management
  browser: Browser;        // Puppeteer browser instance
  page: Page;             // Active page instance
  
  // Data Management
  data: Map<string, any>; // Shared data store
  
  // Logging
  logger: Logger;         // Structured logger
  
  // Identification
  workflowId: string;     // Workflow identifier
  executionId: string;    // Unique execution ID
}
```

### Data Sharing

Effects communicate through the context data store:

```typescript
// Producer effect
context.data.set('products', extractedProducts);

// Consumer effect
const products = context.data.get('products');
```

## Data Flow

### Input Flow

```
Environment Variables → Configuration → Workflow Parameters → Effect Parameters
```

### Processing Flow

```
Raw HTML → DOM Selection → Data Extraction → Transformation → Validation → Serialization
```

### Output Flow

```
Serialized Data → Compression → S3 Upload → Result Notification
```

## Error Handling

### Error Categories

1. **Effect Errors**: Failures within effect execution
2. **Network Errors**: Connection and timeout issues
3. **Validation Errors**: Data quality issues
4. **System Errors**: Infrastructure failures

### Error Handling Strategy

```typescript
class EffectError extends Error {
  constructor(
    public effectId: string,
    public effectType: string,
    message: string,
    public cause?: Error
  ) {
    super(message);
    this.name = 'EffectError';
  }
}
```

### Retry Logic

```typescript
async execute<T>(
  operation: () => Promise<T>,
  options: RetryOptions
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 0; attempt < options.maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt < options.maxAttempts - 1) {
        const delay = this.calculateDelay(attempt, options);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  
  throw lastError!;
}
```

## Performance Considerations

### Rate Limiting

Token bucket algorithm implementation:

```typescript
class RateLimiter {
  private tokens: number;
  private lastRefill: number;
  
  async acquire(): Promise<void> {
    while (this.tokens < 1) {
      await this.refillTokens();
      if (this.tokens < 1) {
        await sleep(100);
      }
    }
    this.tokens--;
  }
  
  private refillTokens(): void {
    const now = Date.now();
    const elapsed = now - this.lastRefill;
    const tokensToAdd = elapsed * this.requestsPerSecond / 1000;
    
    this.tokens = Math.min(this.burst, this.tokens + tokensToAdd);
    this.lastRefill = now;
  }
}
```

### Memory Management

- Single browser instance per execution
- Page reuse across effects
- Automatic resource cleanup
- Streaming for large datasets

### Concurrency Control

- Sequential effect execution within dependencies
- Parallel execution for independent effects
- Connection pooling for API requests

## Extension Points

### Adding New Effects

1. Create effect class extending `BaseEffect`:

```typescript
export class MyCustomEffect extends BaseEffect<MyParams, MyInput, MyOutput> {
  async execute(input: MyInput, context: ExecutionContext): Promise<MyOutput> {
    // Implementation
  }
}
```

2. Register in effect factory:

```typescript
registerEffect('my_custom', MyCustomEffect);
```

3. Add TypeScript types:

```typescript
export interface MyParams {
  // Parameter definitions
}
```

### Custom Validators

```typescript
export class CustomValidator implements Validator {
  validate(data: any, schema: Schema): ValidationResult {
    // Validation logic
  }
}
```

### Storage Adapters

```typescript
export interface StorageAdapter {
  upload(data: Buffer, key: string): Promise<void>;
  download(key: string): Promise<Buffer>;
}
```

### Monitoring Hooks

```typescript
export interface ExecutionHook {
  beforeEffect?(effect: Effect): Promise<void>;
  afterEffect?(effect: Effect, result: any): Promise<void>;
  onError?(effect: Effect, error: Error): Promise<void>;
}
```

## Best Practices

### Effect Design

1. **Single Responsibility**: Each effect should do one thing well
2. **Idempotency**: Effects should be safe to retry
3. **Error Recovery**: Handle expected failures gracefully
4. **Logging**: Provide clear, actionable log messages

### Workflow Design

1. **Dependency Minimization**: Reduce coupling between effects
2. **Error Isolation**: Critical vs non-critical effects
3. **Data Validation**: Validate early and often
4. **Progressive Enhancement**: Start simple, add complexity

### Performance Optimization

1. **Selector Optimization**: Use efficient CSS selectors
2. **Batch Operations**: Group similar operations
3. **Caching**: Cache reusable data
4. **Resource Management**: Clean up resources promptly

## Future Enhancements

1. **Parallel Execution**: Execute independent effects concurrently
2. **Dynamic Workflows**: Runtime workflow modification
3. **Effect Marketplace**: Shareable effect library
4. **Visual Workflow Editor**: GUI for workflow creation
5. **Real-time Monitoring**: Live execution dashboard
6. **Machine Learning**: Adaptive selector generation