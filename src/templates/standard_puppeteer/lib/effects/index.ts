// Effects System Core
// This file provides the base classes and interfaces for the effects system

import { Browser, Page } from 'puppeteer';
import { logger } from '../utils/logger';

export interface ExecutionContext {
  browser: Browser;
  page: Page;
  data: Map<string, any>;
  auth?: {
    token?: string;
    headers?: Record<string, string>;
  };
  logger: typeof logger;
  workflowId: string;
  executionId: string;
}

export interface Effect<TParams = any, TInput = any, TOutput = any> {
  id: string;
  type: string;
  params: TParams;
  dependencies: string[];
  
  execute(input: TInput, context: ExecutionContext): Promise<TOutput>;
  validate(): void;
}

export abstract class BaseEffect<TParams, TInput, TOutput> implements Effect<TParams, TInput, TOutput> {
  constructor(
    public readonly id: string,
    public readonly type: string,
    public readonly params: TParams,
    public readonly dependencies: string[] = []
  ) {
    this.validate();
  }
  
  abstract execute(input: TInput, context: ExecutionContext): Promise<TOutput>;
  
  validate(): void {
    if (!this.id) {
      throw new Error('Effect must have an id');
    }
  }
  
  protected log(level: 'info' | 'warn' | 'error' | 'debug', message: string, meta?: any): void {
    logger[level](`[${this.type}:${this.id}] ${message}`, meta);
  }
}

export class EffectError extends Error {
  constructor(
    public readonly effectId: string,
    public readonly effectType: string,
    message: string,
    public override readonly cause?: Error
  ) {
    super(message);
    this.name = 'EffectError';
  }
}

// Re-export all effect implementations
export * from './dom';
export * from './api';
export * from './data';
export * from './storage';
export * from './control';
export * from './cookie';