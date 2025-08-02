// Rate Limiter Implementation
// Token bucket algorithm with burst support

import { logger } from '../utils/logger';

export interface RateLimiterConfig {
  requestsPerSecond: number;
  burst: number;
  perDomain?: Record<string, number>;
}

export class RateLimiter {
  private tokens: number;
  private lastRefill: number;
  private domainTokens: Map<string, { tokens: number; lastRefill: number }> = new Map();
  
  constructor(private config: RateLimiterConfig) {
    this.tokens = config.burst;
    this.lastRefill = Date.now();
    
    logger.info('Rate limiter initialized', {
      requestsPerSecond: config.requestsPerSecond,
      burst: config.burst,
      domains: Object.keys(config.perDomain || {})
    });
  }
  
  async acquire(domain?: string): Promise<void> {
    if (domain && this.config.perDomain?.[domain]) {
      await this.acquireForDomain(domain);
    } else {
      await this.acquireGlobal();
    }
  }
  
  private async acquireGlobal(): Promise<void> {
    while (true) {
      this.refillTokens();
      
      if (this.tokens >= 1) {
        this.tokens -= 1;
        logger.debug('Token acquired', { remaining: Math.floor(this.tokens) });
        return;
      }
      
      // Calculate wait time
      const tokensNeeded = 1 - this.tokens;
      const waitMs = Math.ceil((tokensNeeded / this.config.requestsPerSecond) * 1000);
      
      logger.debug('Rate limit reached, waiting', { waitMs });
      await new Promise(resolve => setTimeout(resolve, waitMs));
    }
  }
  
  private async acquireForDomain(domain: string): Promise<void> {
    const rateLimit = this.config.perDomain![domain];
    
    if (!this.domainTokens.has(domain)) {
      this.domainTokens.set(domain, {
        tokens: this.config.burst,
        lastRefill: Date.now()
      });
    }
    
    const domainState = this.domainTokens.get(domain)!;
    
    while (true) {
      this.refillDomainTokens(domain, domainState, rateLimit || this.config.requestsPerSecond);
      
      if (domainState.tokens >= 1) {
        domainState.tokens -= 1;
        logger.debug(`Token acquired for domain ${domain}`, { 
          remaining: Math.floor(domainState.tokens) 
        });
        return;
      }
      
      // Calculate wait time
      const tokensNeeded = 1 - domainState.tokens;
      const effectiveRateLimit = rateLimit || this.config.requestsPerSecond;
      const waitMs = Math.ceil((tokensNeeded / effectiveRateLimit) * 1000);
      
      logger.debug(`Rate limit reached for domain ${domain}, waiting`, { waitMs });
      await new Promise(resolve => setTimeout(resolve, waitMs));
    }
  }
  
  private refillTokens(): void {
    const now = Date.now();
    const timePassed = (now - this.lastRefill) / 1000; // Convert to seconds
    const tokensToAdd = timePassed * this.config.requestsPerSecond;
    
    this.tokens = Math.min(this.tokens + tokensToAdd, this.config.burst);
    this.lastRefill = now;
  }
  
  private refillDomainTokens(
    _domain: string, 
    state: { tokens: number; lastRefill: number }, 
    rateLimit: number | undefined
  ): void {
    if (!rateLimit) return;
    const now = Date.now();
    const timePassed = (now - state.lastRefill) / 1000; // Convert to seconds
    const tokensToAdd = timePassed * rateLimit;
    
    state.tokens = Math.min(state.tokens + tokensToAdd, this.config.burst);
    state.lastRefill = now;
  }
  
  reset(): void {
    this.tokens = this.config.burst;
    this.lastRefill = Date.now();
    this.domainTokens.clear();
    
    logger.info('Rate limiter reset');
  }
  
  getStats(): {
    globalTokens: number;
    domainStats: Record<string, number>;
  } {
    this.refillTokens();
    
    const domainStats: Record<string, number> = {};
    for (const [domain, state] of this.domainTokens.entries()) {
      const rateLimit = this.config.perDomain?.[domain] || this.config.requestsPerSecond;
      this.refillDomainTokens(domain, state, rateLimit);
      domainStats[domain] = Math.floor(state.tokens);
    }
    
    return {
      globalTokens: Math.floor(this.tokens),
      domainStats
    };
  }
}

// Helper function to extract domain from URL
export function extractDomain(url: string): string | undefined {
  try {
    const urlObj = new URL(url);
    return urlObj.hostname;
  } catch {
    return undefined;
  }
}