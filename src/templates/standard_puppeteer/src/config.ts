// Configuration
// Auto-generated with environment variable support

import * as dotenv from 'dotenv';
dotenv.config();

export const config = {
  name: 'example_shop',
  workflowId: 'ecommerce_product_scraper',
  workflowConfig: {
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
    "credentials": {},
    "token_extraction": null
  }
}
};

export function validateEnvironment(): void {
  const required = [
    'AWS_ACCESS_KEY_ID',
    'AWS_SECRET_ACCESS_KEY',
    'S3_BUCKET',
    'EXAMPLE_SHOP_USERNAME',
    'EXAMPLE_SHOP_PASSWORD',
  ];

  const missing = required.filter(key => !process.env[key]);
  if (missing.length > 0) {
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }
}
