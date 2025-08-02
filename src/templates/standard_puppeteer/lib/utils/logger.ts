// Logger Implementation
// Structured logging with Airflow-compatible output

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: any;
  workflowId?: string;
  executionId?: string;
  effectId?: string;
}

export class Logger {
  private minLevel: LogLevel;
  private structured: boolean;
  
  constructor(
    minLevel: LogLevel = (process.env['LOG_LEVEL'] as LogLevel) || 'info',
    structured: boolean = process.env['NODE_ENV'] === 'production'
  ) {
    this.minLevel = minLevel;
    this.structured = structured;
  }
  
  private shouldLog(level: LogLevel): boolean {
    const levels: LogLevel[] = ['debug', 'info', 'warn', 'error'];
    const minIndex = levels.indexOf(this.minLevel);
    const levelIndex = levels.indexOf(level);
    return levelIndex >= minIndex;
  }
  
  private formatMessage(level: LogLevel, message: string, context?: any): string {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context
    };
    
    if (this.structured) {
      // Structured JSON output for Airflow
      return JSON.stringify(entry);
    } else {
      // Human-readable format for development
      const prefix = `[${entry.timestamp}] [${level.toUpperCase()}]`;
      const contextStr = context ? ` ${JSON.stringify(context)}` : '';
      return `${prefix} ${message}${contextStr}`;
    }
  }
  
  debug(message: string, context?: any): void {
    if (this.shouldLog('debug')) {
      console.log(this.formatMessage('debug', message, context));
    }
  }
  
  info(message: string, context?: any): void {
    if (this.shouldLog('info')) {
      console.log(this.formatMessage('info', message, context));
    }
  }
  
  warn(message: string, context?: any): void {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', message, context));
    }
  }
  
  error(message: string, context?: any): void {
    if (this.shouldLog('error')) {
      console.error(this.formatMessage('error', message, context));
    }
  }
  
  // Special method for Airflow task logging
  airflowLog(event: 'start' | 'progress' | 'complete' | 'error', data: any): void {
    const message = {
      event,
      timestamp: new Date().toISOString(),
      workflow_id: 'ecommerce_product_scraper',
      ...data
    };
    
    // Always output Airflow logs as structured JSON
    console.log(JSON.stringify(message));
  }
}

// Global logger instance
export const logger = new Logger();

// Performance timer utility
export class Timer {
  private startTime: number;
  private marks: Map<string, number> = new Map();
  
  constructor() {
    this.startTime = Date.now();
  }
  
  mark(name: string): void {
    this.marks.set(name, Date.now());
  }
  
  measure(name: string, startMark?: string): number {
    const endTime = Date.now();
    const startTime = startMark ? this.marks.get(startMark) || this.startTime : this.startTime;
    const duration = endTime - startTime;
    
    logger.debug(`Performance: ${name}`, { durationMs: duration });
    return duration;
  }
  
  getAllMeasurements(): Record<string, number> {
    const measurements: Record<string, number> = {};
    const now = Date.now();
    
    measurements['total'] = now - this.startTime;
    
    for (const [name, time] of this.marks.entries()) {
      measurements[name] = time - this.startTime;
    }
    
    return measurements;
  }
}