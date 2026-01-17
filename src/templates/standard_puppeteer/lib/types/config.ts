// Generated TypeScript types

export interface WorkflowConfig {
  rate_limit: RateLimitConfig;
  retry: RetryConfig;
  s3: S3Config;
  browser: BrowserConfig;
  logging: LoggingConfig;
  auth?: AuthConfig;
}

export interface RateLimitConfig {
  requests_per_second: number;
  burst: number;
  per_domain: Record<string, number>;
}

export interface RetryConfig {
  max_attempts: number;
  backoff_ms: number;
  exponential: boolean;
  jitter: boolean;
}

export interface S3Config {
  bucket: string;
  prefix: string;
  region: string;
  endpoint?: string | null;
  access_key_id?: string | null;
  secret_access_key?: string | null;
}

export interface BrowserConfig {
  headless: boolean;
  viewport: {
    width: number;
    height: number;
    device_scale_factor: number;
  };
  user_agent?: string | null;
  args: string[];
  executable_path?: string | null;
}

export interface LoggingConfig {
  level: 'debug' | 'info' | 'warn' | 'error';
  structured: boolean;
  include_screenshots: boolean;
  s3_logs?: string | null;
}

export interface AuthConfig {
  auth_type: 'basic' | 'oauth' | 'api_key' | 'cookie';
  credentials: Record<string, string>;
  token_extraction?: {
    storage_type: 'local_storage' | 'session_storage' | 'cookie';
    token_keys: string[];
  };
}