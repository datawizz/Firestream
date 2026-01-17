// Cookie-to-API Workflow Definition
// Auto-generated from JSON workflow

import { WorkflowDefinition } from '../lib/types';

// Workflow definition
export const cookieApiWorkflow: WorkflowDefinition = {
  "workflow_id": "cookie_api_scraper",
  "description": "Cookie-based API product catalog scraper",
  "version": "1.0.0",
  "config": {
    "rate_limit": {
      "requests_per_second": 2.0,
      "burst": 5,
      "per_domain": {}
    },
    "retry": {
      "max_attempts": 3,
      "backoff_ms": 1000,
      "exponential": true,
      "jitter": true
    },
    "s3": {
      "bucket": "my-data-lake",
      "prefix": "scraped-data/api-products/",
      "region": "us-east-1",
      "endpoint": null,
      "access_key_id": null,
      "secret_access_key": null
    },
    "browser": {
      "headless": true,
      "viewport": {
        "width": 1920,
        "height": 1080,
        "device_scale_factor": 1.0
      },
      "user_agent": null,
      "args": [],
      "executable_path": null
    },
    "logging": {
      "level": "info",
      "structured": false,
      "include_screenshots": false,
      "s3_logs": null
    },
    "auth": {
      "auth_type": "cookie",
      "credentials": {}
    }
  },
  "effects": [
    {
      "type": "navigate",
      "navigate": {
        "id": "goto_home",
        "dependencies": [],
        "params": {
          "url": "http://localhost:3000",
          "wait_until": "networkidle2",
          "timeout": 30000
        }
      }
    },
    {
      "type": "click",
      "click": {
        "id": "open_login",
        "dependencies": ["goto_home"],
        "params": {
          "selector": ".login-button",
          "wait_for_navigation": true
        }
      }
    },
    {
      "type": "type",
      "type_effect": {
        "id": "enter_credentials",
        "dependencies": ["open_login"],
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
    },
    {
      "type": "click",
      "click": {
        "id": "submit_login",
        "dependencies": ["enter_credentials"],
        "params": {
          "selector": "#login-submit",
          "wait_for_navigation": true
        }
      }
    },
    {
      "type": "wait_for_selector",
      "wait_for_selector": {
        "id": "wait_dashboard",
        "dependencies": ["submit_login"],
        "params": {
          "selector": ".dashboard",
          "timeout": 10000,
          "visible": true
        }
      }
    },
    {
      "type": "extract_cookies",
      "extract_cookies": {
        "id": "extract_session_cookies",
        "dependencies": ["wait_dashboard"],
        "params": {
          "cookie_names": ["sessionId", "authToken"],
          "include_all": false,
          "domain_filter": "localhost"
        }
      }
    },
    {
      "type": "api_request",
      "api_request": {
        "id": "api_get_categories",
        "dependencies": ["extract_session_cookies"],
        "params": {
          "endpoint": "http://localhost:3001/api/categories",
          "method": "GET",
          "cookies_from_context": true,
          "cookie_effect_id": "extract_session_cookies"
        }
      }
    },
    {
      "type": "api_request",
      "api_request": {
        "id": "api_get_stats",
        "dependencies": ["extract_session_cookies"],
        "params": {
          "endpoint": "http://localhost:3001/api/stats",
          "method": "GET",
          "cookies_from_context": true,
          "cookie_effect_id": "extract_session_cookies"
        }
      }
    },
    {
      "type": "api_request",
      "api_request": {
        "id": "api_get_products_page1",
        "dependencies": ["extract_session_cookies"],
        "params": {
          "endpoint": "http://localhost:3001/api/products",
          "method": "GET",
          "params": {
            "page": 1,
            "limit": 50
          },
          "cookies_from_context": true,
          "cookie_effect_id": "extract_session_cookies"
        }
      }
    },
    {
      "type": "api_request",
      "api_request": {
        "id": "api_get_products_page2",
        "dependencies": ["api_get_products_page1"],
        "params": {
          "endpoint": "http://localhost:3001/api/products",
          "method": "GET",
          "params": {
            "page": 2,
            "limit": 50
          },
          "cookies_from_context": true,
          "cookie_effect_id": "extract_session_cookies"
        }
      }
    },
    {
      "type": "api_request",
      "api_request": {
        "id": "api_get_instock_products",
        "dependencies": ["api_get_products_page1"],
        "params": {
          "endpoint": "http://localhost:3001/api/products",
          "method": "GET",
          "params": {
            "inStock": "true",
            "limit": 100
          },
          "cookies_from_context": true,
          "cookie_effect_id": "extract_session_cookies"
        }
      }
    },
    {
      "type": "validate",
      "validate": {
        "id": "validate_api_data",
        "dependencies": ["api_get_products_page1", "api_get_products_page2", "api_get_instock_products"],
        "params": {
          "schema": "ApiProduct",
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
    },
    {
      "type": "serialize_json",
      "serialize_json": {
        "id": "serialize_api_data",
        "dependencies": ["validate_api_data"],
        "params": {
          "schema": "ApiProduct",
          "add_metadata": true,
          "pretty": false
        }
      }
    },
    {
      "type": "upload_s3",
      "upload_s3": {
        "id": "upload_api_results",
        "dependencies": ["serialize_api_data"],
        "params": {
          "bucket": "{{ S3_BUCKET }}",
          "key_template": "api-products/{{ date }}/{{ workflow_id }}-{{ timestamp }}.json",
          "content_type": "application/json",
          "compression": "gzip"
        }
      }
    }
  ],
  "schemas": {
    "ApiProduct": {
      "name": "ApiProduct",
      "fields": [
        {
          "name": "id",
          "field_type": "string",
          "required": true,
          "description": "Product ID",
          "default_value": null,
          "validation": null
        },
        {
          "name": "name",
          "field_type": "string",
          "required": true,
          "description": "Product name",
          "default_value": null,
          "validation": null
        },
        {
          "name": "price",
          "field_type": "number",
          "required": false,
          "description": "Product price",
          "default_value": null,
          "validation": {
            "min_length": null,
            "max_length": null,
            "min_value": 0.0,
            "max_value": null,
            "pattern": null,
            "enum_values": null
          }
        },
        {
          "name": "inStock",
          "field_type": "boolean",
          "required": false,
          "description": "Stock availability",
          "default_value": "true",
          "validation": null
        },
        {
          "name": "category",
          "field_type": "string",
          "required": false,
          "description": "Product category",
          "default_value": null,
          "validation": null
        },
        {
          "name": "description",
          "field_type": "string",
          "required": false,
          "description": "Product description",
          "default_value": null,
          "validation": null
        },
        {
          "name": "rating",
          "field_type": "string",
          "required": false,
          "description": "Product rating",
          "default_value": null,
          "validation": null
        },
        {
          "name": "reviews",
          "field_type": "number",
          "required": false,
          "description": "Number of reviews",
          "default_value": null,
          "validation": null
        }
      ],
      "description": "E-commerce product data from API"
    }
  }
};