// Retry Manager
// Handles retry logic with exponential backoff and jitter

import { logger } from '../utils/logger';

export interface RetryOptions {
  maxAttempts: number;
  backoffMs: number;
  exponential?: boolean;
  jitter?: boolean;
  retryableErrors?: (error: Error) => boolean;
}

export class RetryManager {
  private defaultOptions: RetryOptions = {
    maxAttempts: 3,
    backoffMs: 1000,
    exponential: true,
    jitter: true,
    retryableErrors: (error) => {
      // Default: retry on network errors and timeouts
      const message = error.message.toLowerCase();
      return (
        message.includes('timeout') ||
        message.includes('network') ||
        message.includes('econnrefused') ||
        message.includes('econnreset') ||
        message.includes('etimedout') ||
        message.includes('navigation failed')
      );
    }
  };
  
  constructor(defaultOptions?: Partial<RetryOptions>) {
    if (defaultOptions) {
      this.defaultOptions = { ...this.defaultOptions, ...defaultOptions };
    }
    
    logger.info('Retry manager initialized', {
      maxAttempts: this.defaultOptions.maxAttempts,
      backoffMs: this.defaultOptions.backoffMs,
      exponential: this.defaultOptions.exponential
    });
  }
  
  async execute<T>(
    operation: () => Promise<T>,
    options?: Partial<RetryOptions>
  ): Promise<T> {
    const opts = { ...this.defaultOptions, ...options };
    let lastError: Error | undefined;
    
    for (let attempt = 1; attempt <= opts.maxAttempts; attempt++) {
      try {
        logger.debug(`Executing operation, attempt ${attempt}/${opts.maxAttempts}`);
        const result = await operation();
        
        if (attempt > 1) {
          logger.info(`Operation succeeded on attempt ${attempt}`);
        }
        
        return result;
      } catch (error) {
        lastError = error as Error;
        
        logger.warn(`Operation failed on attempt ${attempt}`, {
          error: lastError.message,
          retriesRemaining: opts.maxAttempts - attempt
        });
        
        // Check if error is retryable
        if (opts.retryableErrors && !opts.retryableErrors(lastError)) {
          logger.error('Error is not retryable', { error: lastError.message });
          throw lastError;
        }
        
        // Don't wait after the last attempt
        if (attempt < opts.maxAttempts) {
          const waitTime = this.calculateBackoff(attempt, opts);
          logger.info(`Waiting ${waitTime}ms before retry`);
          await this.wait(waitTime);
        }
      }
    }
    
    logger.error(`All ${opts.maxAttempts} attempts failed`);
    throw new RetryError(
      `Operation failed after ${opts.maxAttempts} attempts`,
      lastError!,
      opts.maxAttempts
    );
  }
  
  private calculateBackoff(attempt: number, options: RetryOptions): number {
    let backoff = options.backoffMs;
    
    if (options.exponential) {
      // Exponential backoff: base * 2^(attempt-1)
      backoff = options.backoffMs * Math.pow(2, attempt - 1);
    }
    
    if (options.jitter) {
      // Add random jitter ±25%
      const jitterRange = backoff * 0.25;
      const jitter = (Math.random() - 0.5) * 2 * jitterRange;
      backoff += jitter;
    }
    
    // Cap at 30 seconds
    return Math.min(Math.max(backoff, 0), 30000);
  }
  
  private wait(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

export class RetryError extends Error {
  constructor(
    message: string,
    public override readonly cause: Error,
    public readonly attempts: number
  ) {
    super(message);
    this.name = 'RetryError';
  }
}

// Retry strategies for common scenarios
export const RetryStrategies = {
  // Conservative: fewer retries, longer backoff
  conservative: (): Partial<RetryOptions> => ({
    maxAttempts: 2,
    backoffMs: 2000,
    exponential: true,
    jitter: true
  }),
  
  // Aggressive: more retries, shorter backoff
  aggressive: (): Partial<RetryOptions> => ({
    maxAttempts: 5,
    backoffMs: 500,
    exponential: false,
    jitter: true
  }),
  
  // Network issues: exponential backoff with jitter
  network: (): Partial<RetryOptions> => ({
    maxAttempts: 4,
    backoffMs: 1000,
    exponential: true,
    jitter: true,
    retryableErrors: (error) => {
      const message = error.message.toLowerCase();
      return (
        message.includes('network') ||
        message.includes('timeout') ||
        message.includes('econnrefused') ||
        message.includes('econnreset') ||
        message.includes('dns')
      );
    }
  }),
  
  // Rate limiting: longer backoff
  rateLimit: (): Partial<RetryOptions> => ({
    maxAttempts: 3,
    backoffMs: 5000,
    exponential: true,
    jitter: false,
    retryableErrors: (error) => {
      const message = error.message.toLowerCase();
      return (
        message.includes('rate limit') ||
        message.includes('429') ||
        message.includes('too many requests')
      );
    }
  })
};