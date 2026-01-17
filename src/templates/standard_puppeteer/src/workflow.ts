// Workflow Definition
// Auto-generated from JSON workflow

import { WorkflowDefinition } from '../lib/types';

// Workflow definition
export const workflow: WorkflowDefinition = {
  "workflow_id": "ecommerce_product_scraper",
  "description": "E-commerce product catalog scraper",
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
      "prefix": "scraped-data/products/",
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
      "auth_type": "basic",
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
          "url": "https://shop.example.com",
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
      "type": "navigate",
      "navigate": {
        "id": "goto_products",
        "dependencies": ["wait_dashboard"],
        "params": {
          "url": "https://shop.example.com/products",
          "wait_until": "networkidle2"
        }
      }
    },
    {
      "type": "extract_all",
      "extract_all": {
        "id": "extract_products",
        "dependencies": ["goto_products"],
        "params": {
          "selector": ".product-item",
          "fields": {
            "id": {
              "extraction_type": "attribute",
              "selector": "[data-product-id]",
              "attribute": "data-product-id"
            },
            "inStock": {
              "extraction_type": "text",
              "selector": ".availability",
              "transform": "text => text.toLowerCase().includes('in stock')",
              "default_value": "false"
            },
            "price": {
              "extraction_type": "text",
              "selector": ".product-price",
              "transform": "parseFloat",
              "default_value": "0"
            },
            "name": {
              "extraction_type": "text",
              "selector": ".product-name"
            }
          },
          "pagination": {
            "pagination_type": "click_next",
            "next_selector": ".pagination .next-page",
            "max_pages": 5,
            "delay_ms": 1000
          }
        }
      }
    },
    {
      "type": "validate",
      "validate": {
        "id": "validate_products",
        "dependencies": ["extract_products"],
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
    },
    {
      "type": "serialize_json",
      "serialize_json": {
        "id": "serialize_data",
        "dependencies": ["validate_products"],
        "params": {
          "schema": "Product",
          "add_metadata": true,
          "pretty": false
        }
      }
    },
    {
      "type": "upload_s3",
      "upload_s3": {
        "id": "upload_results",
        "dependencies": ["serialize_data"],
        "params": {
          "bucket": "{{ S3_BUCKET }}",
          "key_template": "products/{{ date }}/{{ workflow_id }}-{{ timestamp }}.json",
          "content_type": "application/json",
          "compression": "gzip"
        }
      }
    }
  ],
  "schemas": {
    "Product": {
      "name": "Product",
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
        }
      ],
      "description": "E-commerce product data"
    }
  }
};