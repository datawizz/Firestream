// Generated TypeScript types

// HTTP Method types
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

// Compression types
export type CompressionType = 'none' | 'gzip' | 'brotli';

// DOM Effect Types
export interface NavigateParams {
  url: string;
  wait_until?: 'load' | 'domcontentloaded' | 'networkidle0' | 'networkidle2';
  timeout?: number;
}

export interface ClickParams {
  selector: string;
  wait_for_navigation?: boolean;
  click_count?: number;
}

export interface TypeParams {
  fields: Array<{
    selector: string;
    value: string;
    clear_first?: boolean;
  }>;
  delay?: number;
}

export interface WaitForSelectorParams {
  selector: string;
  timeout?: number;
  visible?: boolean;
}

export interface ExtractAllParams {
  selector: string;
  fields: Record<string, ExtractionRule>;
  pagination?: PaginationConfig;
}

export interface ExtractionRule {
  selector: string;
  extraction_type: 'text' | 'attribute' | 'html' | 'value';
  attribute?: string;
  transform?: string;
  default_value?: any;
}

export interface PaginationConfig {
  pagination_type: 'click_next' | 'infinite_scroll';
  next_selector?: string;
  max_pages?: number;
  delay_ms?: number;
}

export interface ExtractTextParams {
  selector: string;
  multiple?: boolean;
}

export interface ScreenshotParams {
  filename: string;
  full_page?: boolean;
  upload_to_s3?: boolean;
}

// API Effect Types
export interface ApiRequestParams {
  endpoint: string;
  method: HttpMethod;
  headers?: Record<string, string>;
  params?: Record<string, any>;
  data?: any;
  auth_from_context?: boolean;
  cookies_from_context?: boolean;
  cookie_effect_id?: string;
}

export interface ExtractAuthTokenParams {
  storage_type: 'local_storage' | 'session_storage' | 'cookie';
  token_keys: string[];
}

// Cookie Management Effect Types
export interface ExtractCookiesParams {
  cookie_names?: string[];
  include_all?: boolean;
  domain_filter?: string;
}

export interface SetCookieParams {
  name: string;
  value: string;
  domain?: string;
  path?: string;
  secure?: boolean;
  httpOnly?: boolean;
  maxAge?: number;
}

export interface ClearCookiesParams {
  cookie_names?: string[];
  clear_all?: boolean;
  domain_filter?: string;
}

// Control Flow Effect Types
export interface DelayParams {
  milliseconds: number;
  jitter?: number;
}

export interface ConditionalParams {
  condition: ConditionExpression;
  then_effects: string[];
  else_effects: string[];
}

export interface ConditionExpression {
  expression_type: 'context_value' | 'effect_result' | 'selector_exists';
  field: string;
  operator: 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'contains' | 'exists';
  value?: any;
}

export interface RetryParams {
  effect_id: string;
  max_attempts: number;
  backoff_ms: number;
}

// Data Processing Effect Types
export interface ValidateParams {
  schema: string;
  rules: ValidationRule[];
  fail_on_error?: boolean;
}

export interface ValidationRule {
  field: string;
  rule_type: 'required' | 'min' | 'max' | 'pattern' | 'custom';
  value?: string | number;
  message?: string;
}

export interface SerializeJsonParams {
  schema: string;
  add_metadata?: boolean;
  pretty?: boolean;
}

export interface TransformParams {
  transformations: Array<{
    field: string;
    operation: 'lowercase' | 'uppercase' | 'trim' | 'parse_number' | 'parse_date';
  }>;
}

// Storage Effect Types
export interface UploadS3Params {
  bucket: string;
  key_template: string;
  content_type?: string;
  compression?: CompressionType;
  acl?: string;
}

export interface SaveLocalParams {
  filename: string;
  directory?: string;
}

// Effect Definition Union Type
export interface EffectDefinition {
  type: 'navigate' | 'click' | 'type' | 'wait_for_selector' | 'extract_all' | 
        'extract_text' | 'screenshot' | 'api_request' | 'extract_auth_token' | 
        'extract_cookies' | 'set_cookie' | 'clear_cookies' |
        'delay' | 'conditional' | 'retry' | 'validate' | 'serialize_json' | 
        'transform' | 'upload_s3' | 'save_local';
  
  navigate?: {
    id: string;
    params: NavigateParams;
    dependencies?: string[];
  };
  
  click?: {
    id: string;
    params: ClickParams;
    dependencies?: string[];
  };
  
  type_effect?: {
    id: string;
    params: TypeParams;
    dependencies?: string[];
  };
  
  wait_for_selector?: {
    id: string;
    params: WaitForSelectorParams;
    dependencies?: string[];
  };
  
  extract_all?: {
    id: string;
    params: ExtractAllParams;
    dependencies?: string[];
  };
  
  extract_text?: {
    id: string;
    params: ExtractTextParams;
    dependencies?: string[];
  };
  
  screenshot?: {
    id: string;
    params: ScreenshotParams;
    dependencies?: string[];
  };
  
  api_request?: {
    id: string;
    params: ApiRequestParams;
    dependencies?: string[];
  };
  
  extract_auth_token?: {
    id: string;
    params: ExtractAuthTokenParams;
    dependencies?: string[];
  };
  
  extract_cookies?: {
    id: string;
    params: ExtractCookiesParams;
    dependencies?: string[];
  };
  
  set_cookie?: {
    id: string;
    params: SetCookieParams;
    dependencies?: string[];
  };
  
  clear_cookies?: {
    id: string;
    params: ClearCookiesParams;
    dependencies?: string[];
  };
  
  delay?: {
    id: string;
    params: DelayParams;
    dependencies?: string[];
  };
  
  conditional?: {
    id: string;
    params: ConditionalParams;
    dependencies?: string[];
  };
  
  retry?: {
    id: string;
    params: RetryParams;
    dependencies?: string[];
  };
  
  validate?: {
    id: string;
    params: ValidateParams;
    dependencies?: string[];
  };
  
  serialize_json?: {
    id: string;
    params: SerializeJsonParams;
    dependencies?: string[];
  };
  
  transform?: {
    id: string;
    params: TransformParams;
    dependencies?: string[];
  };
  
  upload_s3?: {
    id: string;
    params: UploadS3Params;
    dependencies?: string[];
  };
  
  save_local?: {
    id: string;
    params: SaveLocalParams;
    dependencies?: string[];
  };
}

// Enhanced Effect Type Definitions
// Provides better type safety for effect creation and execution

import { Effect } from '../effects';

// Map effect types to their parameter types
export interface EffectParamMap {
  navigate: NavigateParams;
  click: ClickParams;
  type: TypeParams;
  wait_for_selector: WaitForSelectorParams;
  extract_all: ExtractAllParams;
  extract_text: ExtractTextParams;
  screenshot: ScreenshotParams;
  api_request: ApiRequestParams;
  extract_auth_token: ExtractAuthTokenParams;
  extract_cookies: ExtractCookiesParams;
  set_cookie: SetCookieParams;
  clear_cookies: ClearCookiesParams;
  delay: DelayParams;
  conditional: ConditionalParams;
  retry: RetryParams;
  validate: ValidateParams;
  serialize_json: SerializeJsonParams;
  transform: TransformParams;
  upload_s3: UploadS3Params;
  save_local: SaveLocalParams;
}

// Map effect types to their input types
export interface EffectInputMap {
  navigate: void;
  click: void;
  type: void;
  wait_for_selector: void;
  extract_all: void;
  extract_text: void;
  screenshot: void;
  api_request: any;
  extract_auth_token: void;
  extract_cookies: void;
  set_cookie: void;
  clear_cookies: void;
  delay: void;
  conditional: any;
  retry: any;
  validate: any[];
  serialize_json: any[];
  transform: any[];
  upload_s3: string;
  save_local: string;
}

// Map effect types to their output types
export interface EffectOutputMap {
  navigate: void;
  click: void;
  type: void;
  wait_for_selector: void;
  extract_all: any[];
  extract_text: string | string[];
  screenshot: string;
  api_request: any;
  extract_auth_token: string | null;
  extract_cookies: Array<{ name: string; value: string; domain: string; path: string }>;
  set_cookie: void;
  clear_cookies: number;
  delay: void;
  conditional: { branch: 'then' | 'else'; effects: string[] };
  retry: any;
  validate: any[];
  serialize_json: string;
  transform: any[];
  upload_s3: { bucket: string; key: string; size: number };
  save_local: string;
}

// Type helper to get effect types
export type EffectType = keyof EffectParamMap;

// Type helper to get typed effect
export type TypedEffect<T extends EffectType> = Effect<
  EffectParamMap[T],
  EffectInputMap[T],
  EffectOutputMap[T]
>;

// Type guard functions
export function isNavigateParams(params: any): params is NavigateParams {
  return params && typeof params.url === 'string';
}

export function isClickParams(params: any): params is ClickParams {
  return params && typeof params.selector === 'string';
}

export function isTypeParams(params: any): params is TypeParams {
  return params && Array.isArray(params.fields);
}

export function isWaitForSelectorParams(params: any): params is WaitForSelectorParams {
  return params && typeof params.selector === 'string';
}

export function isExtractAllParams(params: any): params is ExtractAllParams {
  return params && typeof params.selector === 'string' && params.fields;
}

export function isValidateParams(params: any): params is ValidateParams {
  return params && typeof params.schema === 'string' && Array.isArray(params.rules);
}

export function isSerializeJsonParams(params: any): params is SerializeJsonParams {
  return params && typeof params.schema === 'string';
}

export function isUploadS3Params(params: any): params is UploadS3Params {
  return params && typeof params.bucket === 'string' && typeof params.key_template === 'string';
}

// Type guard for effect definitions
export function getEffectType(definition: any): EffectType | null {
  if (!definition || !definition.type) return null;

  const validTypes: EffectType[] = [
    'navigate', 'click', 'type', 'wait_for_selector', 'extract_all',
    'extract_text', 'screenshot', 'api_request', 'extract_auth_token',
    'extract_cookies', 'set_cookie', 'clear_cookies',
    'delay', 'conditional', 'retry', 'validate', 'serialize_json',
    'transform', 'upload_s3', 'save_local'
  ];

  return validTypes.includes(definition.type) ? definition.type as EffectType : null;
}

// Helper to validate effect parameters
export function validateEffectParams<T extends EffectType>(
  type: T,
  params: any
): params is EffectParamMap[T] {
  switch (type) {
    case 'navigate':
      return isNavigateParams(params);
    case 'click':
      return isClickParams(params);
    case 'type':
      return isTypeParams(params);
    case 'wait_for_selector':
      return isWaitForSelectorParams(params);
    case 'extract_all':
      return isExtractAllParams(params);
    case 'validate':
      return isValidateParams(params);
    case 'serialize_json':
      return isSerializeJsonParams(params);
    case 'upload_s3':
      return isUploadS3Params(params);
    default:
      // For other types, just check if params exist
      return params !== null && params !== undefined;
  }
}