# E-commerce Scraper API Reference

## Table of Contents

1. [DOM Manipulation Effects](#dom-manipulation-effects)
2. [API Effects](#api-effects)
3. [Control Flow Effects](#control-flow-effects)
4. [Data Processing Effects](#data-processing-effects)
5. [Storage Effects](#storage-effects)
6. [Configuration Reference](#configuration-reference)
7. [Schema Definitions](#schema-definitions)
8. [Context API](#context-api)
9. [Type Definitions](#type-definitions)

## DOM Manipulation Effects

### navigate

Navigates to a specified URL.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `url` | string | Yes | - | The URL to navigate to |
| `wait_until` | string | No | `networkidle2` | When to consider navigation complete: `load`, `domcontentloaded`, `networkidle0`, `networkidle2` |
| `timeout` | number | No | 30000 | Maximum time to wait in milliseconds |

**Example:**

```json
{
  "type": "navigate",
  "id": "goto_home",
  "params": {
    "url": "https://shop.example.com",
    "wait_until": "networkidle2",
    "timeout": 30000
  }
}
```

### click

Clicks on an element matching the selector.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `selector` | string | Yes | - | CSS selector for the element to click |
| `wait_for_navigation` | boolean | No | false | Wait for navigation after click |
| `click_count` | number | No | 1 | Number of clicks |

**Example:**

```json
{
  "type": "click",
  "id": "submit_login",
  "params": {
    "selector": "#login-submit",
    "wait_for_navigation": true
  }
}
```

### type

Types text into form fields.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `fields` | Array | Yes | - | Array of field configurations |
| `fields[].selector` | string | Yes | - | CSS selector for the input field |
| `fields[].value` | string | Yes | - | Text to type (supports template variables) |
| `fields[].clear_first` | boolean | No | false | Clear field before typing |
| `delay` | number | No | 50 | Delay between keystrokes in milliseconds |

**Example:**

```json
{
  "type": "type",
  "id": "enter_credentials",
  "params": {
    "fields": [
      {
        "selector": "#username",
        "value": "{{ SHOP_USERNAME }}",
        "clear_first": true
      },
      {
        "selector": "#password",
        "value": "{{ SHOP_PASSWORD }}",
        "clear_first": true
      }
    ],
    "delay": 50
  }
}
```

### wait_for_selector

Waits for an element to appear on the page.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `selector` | string | Yes | - | CSS selector to wait for |
| `timeout` | number | No | 30000 | Maximum wait time in milliseconds |
| `visible` | boolean | No | true | Wait for element to be visible |

**Example:**

```json
{
  "type": "wait_for_selector",
  "id": "wait_dashboard",
  "params": {
    "selector": ".dashboard",
    "timeout": 10000,
    "visible": true
  }
}
```

### extract_all

Extracts data from multiple elements matching a selector.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `selector` | string | Yes | - | CSS selector for container elements |
| `fields` | Object | Yes | - | Field extraction rules |
| `pagination` | Object | No | - | Pagination configuration |

**Field Extraction Rules:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `type` | string | Yes | - | Extraction type: `text`, `attribute`, `html`, `value` |
| `selector` | string | Yes | - | CSS selector within container |
| `attribute` | string | No* | - | Attribute name (required for `attribute` type) |
| `transform` | string | No | - | Transform function: `parseFloat`, `parseInt`, or inline function |
| `default_value` | any | No | null | Default value if extraction fails |

**Pagination Configuration:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `type` | string | Yes | - | Pagination type: `click_next`, `infinite_scroll` |
| `next_selector` | string | No* | - | Next button selector (for `click_next`) |
| `max_pages` | number | No | 1 | Maximum pages to process |
| `delay_ms` | number | No | 1000 | Delay between pages in milliseconds |

**Example:**

```json
{
  "type": "extract_all",
  "id": "extract_products",
  "params": {
    "selector": ".product-item",
    "fields": {
      "id": {
        "type": "attribute",
        "selector": "[data-product-id]",
        "attribute": "data-product-id"
      },
      "name": {
        "type": "text",
        "selector": ".product-name"
      },
      "price": {
        "type": "text",
        "selector": ".product-price",
        "transform": "parseFloat"
      },
      "inStock": {
        "type": "text",
        "selector": ".availability",
        "transform": "text => text.toLowerCase().includes('in stock')",
        "default_value": false
      }
    },
    "pagination": {
      "type": "click_next",
      "next_selector": ".pagination .next-page",
      "max_pages": 5,
      "delay_ms": 1000
    }
  }
}
```

### extract_text

Extracts text content from elements.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `selector` | string | Yes | - | CSS selector for elements |
| `multiple` | boolean | No | false | Extract from all matching elements |

**Example:**

```json
{
  "type": "extract_text",
  "id": "get_page_title",
  "params": {
    "selector": "h1",
    "multiple": false
  }
}
```

### screenshot

Takes a screenshot of the page.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `filename` | string | Yes | - | Screenshot filename |
| `full_page` | boolean | No | false | Capture full scrollable page |
| `upload_to_s3` | boolean | No | false | Upload screenshot to S3 |

**Example:**

```json
{
  "type": "screenshot",
  "id": "capture_products",
  "params": {
    "filename": "products-page.png",
    "full_page": true,
    "upload_to_s3": true
  }
}
```

## API Effects

### api_request

Makes HTTP API requests.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `endpoint` | string | Yes | - | API endpoint URL |
| `method` | string | Yes | - | HTTP method: `GET`, `POST`, `PUT`, `DELETE`, `PATCH` |
| `headers` | Object | No | {} | Request headers |
| `params` | Object | No | {} | Query parameters |
| `data` | any | No | - | Request body |
| `auth_from_context` | boolean | No | false | Use auth tokens from context |

**Example:**

```json
{
  "type": "api_request",
  "id": "fetch_additional_data",
  "params": {
    "endpoint": "https://api.example.com/products",
    "method": "GET",
    "headers": {
      "Accept": "application/json"
    },
    "params": {
      "limit": 100,
      "offset": 0
    }
  }
}
```

### extract_auth_token

Extracts authentication tokens from browser storage.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `storage_type` | string | Yes | - | Storage type: `local_storage`, `session_storage`, `cookie` |
| `token_keys` | string[] | Yes | - | Array of token keys to extract |

**Example:**

```json
{
  "type": "extract_auth_token",
  "id": "get_session_token",
  "params": {
    "storage_type": "local_storage",
    "token_keys": ["auth_token", "refresh_token"]
  }
}
```

## Control Flow Effects

### delay

Adds a delay between effects.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `milliseconds` | number | Yes | - | Delay duration in milliseconds |
| `jitter` | number | No | 0 | Random jitter to add (0-1) |

**Example:**

```json
{
  "type": "delay",
  "id": "wait_before_next",
  "params": {
    "milliseconds": 2000,
    "jitter": 0.2
  }
}
```

### conditional

Executes effects based on conditions.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `condition` | Object | Yes | - | Condition expression |
| `then_effects` | string[] | Yes | - | Effect IDs to execute if true |
| `else_effects` | string[] | No | [] | Effect IDs to execute if false |

**Condition Expression:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `expression_type` | string | Yes | - | Type: `context_value`, `effect_result`, `selector_exists` |
| `field` | string | Yes | - | Field to evaluate |
| `operator` | string | Yes | - | Operator: `equals`, `not_equals`, `greater_than`, `less_than`, `contains`, `exists` |
| `value` | any | No | - | Value to compare against |

**Example:**

```json
{
  "type": "conditional",
  "id": "check_login_status",
  "params": {
    "condition": {
      "expression_type": "selector_exists",
      "field": ".user-menu",
      "operator": "exists"
    },
    "then_effects": ["proceed_to_products"],
    "else_effects": ["retry_login"]
  }
}
```

### retry

Retries a failed effect.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `effect_id` | string | Yes | - | ID of effect to retry |
| `max_attempts` | number | Yes | - | Maximum retry attempts |
| `backoff_ms` | number | Yes | - | Base backoff delay in milliseconds |

**Example:**

```json
{
  "type": "retry",
  "id": "retry_extraction",
  "params": {
    "effect_id": "extract_products",
    "max_attempts": 3,
    "backoff_ms": 1000
  }
}
```

## Data Processing Effects

### validate

Validates data against schema and rules.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `schema` | string | Yes | - | Schema name to validate against |
| `rules` | Array | Yes | - | Validation rules |
| `fail_on_error` | boolean | No | false | Fail workflow on validation error |

**Validation Rules:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `field` | string | Yes | - | Field to validate |
| `rule_type` | string | Yes | - | Rule type: `required`, `min`, `max`, `pattern`, `custom` |
| `value` | any | No | - | Value for comparison |
| `message` | string | No | - | Error message |

**Example:**

```json
{
  "type": "validate",
  "id": "validate_products",
  "params": {
    "schema": "Product",
    "rules": [
      {
        "field": "id",
        "rule_type": "required",
        "message": "Product ID is required"
      },
      {
        "field": "price",
        "rule_type": "min",
        "value": 0,
        "message": "Price must be non-negative"
      }
    ],
    "fail_on_error": false
  }
}
```

### serialize_json

Serializes data to JSON format.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `schema` | string | Yes | - | Schema name for serialization |
| `add_metadata` | boolean | No | true | Include metadata in output |
| `pretty` | boolean | No | false | Pretty print JSON |

**Example:**

```json
{
  "type": "serialize_json",
  "id": "serialize_data",
  "params": {
    "schema": "Product",
    "add_metadata": true,
    "pretty": false
  }
}
```

### transform

Transforms data fields.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `transformations` | Array | Yes | - | Array of transformations |
| `transformations[].field` | string | Yes | - | Field to transform |
| `transformations[].operation` | string | Yes | - | Operation: `lowercase`, `uppercase`, `trim`, `parse_number`, `parse_date` |

**Example:**

```json
{
  "type": "transform",
  "id": "normalize_data",
  "params": {
    "transformations": [
      {
        "field": "name",
        "operation": "trim"
      },
      {
        "field": "category",
        "operation": "lowercase"
      }
    ]
  }
}
```

## Storage Effects

### upload_s3

Uploads data to Amazon S3.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `bucket` | string | Yes | - | S3 bucket name (supports templates) |
| `key_template` | string | Yes | - | S3 key template |
| `content_type` | string | No | `application/json` | MIME type |
| `compression` | string | No | `none` | Compression: `none`, `gzip`, `brotli` |
| `acl` | string | No | - | S3 ACL setting |

**Template Variables:**

- `{{ date }}` - Current date (YYYY-MM-DD)
- `{{ timestamp }}` - Unix timestamp
- `{{ workflow_id }}` - Workflow ID
- `{{ execution_id }}` - Execution ID

**Example:**

```json
{
  "type": "upload_s3",
  "id": "upload_results",
  "params": {
    "bucket": "{{ S3_BUCKET }}",
    "key_template": "products/{{ date }}/{{ workflow_id }}-{{ timestamp }}.json",
    "content_type": "application/json",
    "compression": "gzip"
  }
}
```

### save_local

Saves data to local filesystem.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `filename` | string | Yes | - | Output filename |
| `directory` | string | No | `./output` | Output directory |

**Example:**

```json
{
  "type": "save_local",
  "id": "save_backup",
  "params": {
    "filename": "products-backup.json",
    "directory": "./backups"
  }
}
```

## Configuration Reference

### Workflow Configuration

```json
{
  "workflow_id": "string",
  "metadata": {
    "site_name": "string",
    "version": "string",
    "airflow_task_id": "string",
    "description": "string",
    "author": "string",
    "tags": ["string"]
  },
  "config": {
    "rate_limit": {
      "requests_per_second": 2.0,
      "burst": 5,
      "per_domain": {
        "example.com": 1.0
      }
    },
    "retry": {
      "max_attempts": 3,
      "backoff_ms": 1000,
      "exponential": true,
      "jitter": true
    },
    "s3": {
      "bucket": "string",
      "prefix": "string",
      "region": "string",
      "endpoint": "string",
      "access_key_id": "string",
      "secret_access_key": "string"
    },
    "browser": {
      "headless": true,
      "viewport": {
        "width": 1920,
        "height": 1080,
        "device_scale_factor": 1.0
      },
      "user_agent": "string",
      "args": ["--no-sandbox"],
      "executable_path": "string"
    },
    "logging": {
      "level": "info",
      "structured": true,
      "include_screenshots": false,
      "s3_logs": "bucket/path"
    },
    "auth": {
      "auth_type": "basic",
      "credentials": {},
      "token_extraction": {}
    }
  }
}
```

## Schema Definitions

### Schema Structure

```json
{
  "schemas": {
    "SchemaName": {
      "name": "string",
      "description": "string",
      "fields": [
        {
          "name": "string",
          "field_type": "string|number|boolean|array|object",
          "required": true,
          "description": "string",
          "default_value": "any",
          "validation": {
            "min_length": 0,
            "max_length": 100,
            "min_value": 0,
            "max_value": 1000,
            "pattern": "regex",
            "enum_values": ["value1", "value2"]
          }
        }
      ]
    }
  }
}
```

### Built-in Types

- `string`: Text data
- `number`: Numeric data (integer or float)
- `boolean`: True/false values
- `array`: Arrays of values
- `object`: Nested objects

## Context API

### ExecutionContext

The execution context is available to all effects:

```typescript
interface ExecutionContext {
  // Browser instance
  browser: Browser;
  
  // Current page
  page: Page;
  
  // Shared data store
  data: Map<string, any>;
  
  // Logger instance
  logger: Logger;
  
  // Workflow identifier
  workflowId: string;
  
  // Unique execution ID
  executionId: string;
}
```

### Context Methods

```typescript
// Store data
context.data.set('key', value);

// Retrieve data
const value = context.data.get('key');

// Check existence
if (context.data.has('key')) {
  // ...
}

// Delete data
context.data.delete('key');

// Log message
context.logger.info('Message', { meta: 'data' });
```

## Type Definitions

### Core Types

```typescript
// HTTP Methods
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

// Compression Types
type CompressionType = 'none' | 'gzip' | 'brotli';

// Browser Events
type WaitUntilOption = 'load' | 'domcontentloaded' | 'networkidle0' | 'networkidle2';

// Extraction Types
type ExtractionType = 'text' | 'attribute' | 'html' | 'value';

// Pagination Types
type PaginationType = 'click_next' | 'infinite_scroll';

// Validation Rule Types
type RuleType = 'required' | 'min' | 'max' | 'pattern' | 'custom';

// Condition Operators
type ConditionOperator = 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'contains' | 'exists';
```

### Effect Result Types

```typescript
// Success Result
interface EffectSuccess<T> {
  success: true;
  data: T;
  duration: number;
}

// Error Result
interface EffectError {
  success: false;
  error: {
    message: string;
    type: string;
    stack?: string;
  };
  duration: number;
}

// Combined Result Type
type EffectResult<T> = EffectSuccess<T> | EffectError;
```

### Workflow Result

```typescript
interface WorkflowResult {
  workflow_id: string;
  execution_id: string;
  status: 'completed' | 'failed' | 'partial';
  start_time: Date;
  end_time: Date;
  duration_ms: number;
  data: Record<string, any>;
  errors: WorkflowError[];
  screenshots: string[];
  logs: LogEntry[];
}

interface WorkflowError {
  effect_id: string;
  error_type: string;
  message: string;
  timestamp: Date;
  retry_count: number;
  stack?: string;
}
```