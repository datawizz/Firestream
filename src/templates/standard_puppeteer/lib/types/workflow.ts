// Generated TypeScript types
import { EffectDefinition } from './effects';
import { WorkflowConfig } from './config';

export interface WorkflowDefinition {
  workflow_id: string;
  description: string;
  version: string;
  config: WorkflowConfig;
  effects: EffectDefinition[];
  schemas?: Record<string, any>;
}

export interface WorkflowResult {
  workflow_id: string;
  execution_id: string;
  status: WorkflowStatus;
  start_time: Date;
  end_time?: Date;
  duration_ms?: number;
  data: Record<string, any>;
  errors: WorkflowError[];
  screenshots: string[];
  logs: string[];
}

export interface WorkflowError {
  effect_id: string;
  error_type: string;
  message: string;
  stack?: string;
  timestamp: Date;
  retry_count: number;
}

export type WorkflowStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';