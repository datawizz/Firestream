# E-commerce Scraper Quick Reference

## Common Commands

### Development
```bash
npm run dev                    # Run with hot reload
npm run build                  # Build TypeScript
npm start                      # Run production build
npm test                       # Run unit tests
npm run test:integration       # Run integration tests
npm run lint                   # Run linter
npm run typecheck             # Type check
```

### Make Commands
```bash
make run                       # Build and run scraper
make docker                    # Build Docker image
make docker-run               # Run in Docker
make test                      # Run all tests
make clean                     # Clean build artifacts
make help                      # Show all commands
```

### Testing
```bash
# Run integration tests with visible browser
HEADLESS=false npm run test:integration

# Run specific test
node tests/runTests.js --test=authentication

# Start mock store manually
cd tests/integration/mock-store && npm start
```

## Environment Variables

### Required
```bash
AWS_ACCESS_KEY_ID=your_key
AWS_SECRET_ACCESS_KEY=your_secret
S3_BUCKET=your-bucket
EXAMPLE_SHOP_USERNAME=username
EXAMPLE_SHOP_PASSWORD=password
```

### Optional
```bash
LOG_LEVEL=debug|info|warn|error    # Default: info
HEADLESS=true|false                # Default: true
PUPPETEER_EXECUTABLE_PATH=/path    # Chrome path
RATE_LIMIT_RPS=2                   # Requests per second
```

## Effects Cheat Sheet

### DOM Effects
| Effect | Purpose | Key Parameters |
|--------|---------|----------------|
| `navigate` | Go to URL | `url`, `wait_until` |
| `click` | Click element | `selector`, `wait_for_navigation` |
| `type` | Type text | `fields[]`, `delay` |
| `wait_for_selector` | Wait for element | `selector`, `timeout` |
| `extract_all` | Extract data | `selector`, `fields`, `pagination` |
| `screenshot` | Take screenshot | `filename`, `full_page` |

### Data Effects
| Effect | Purpose | Key Parameters |
|--------|---------|----------------|
| `validate` | Validate data | `schema`, `rules[]` |
| `serialize_json` | Convert to JSON | `schema`, `pretty` |
| `transform` | Transform fields | `transformations[]` |

### Storage Effects
| Effect | Purpose | Key Parameters |
|--------|---------|----------------|
| `upload_s3` | Upload to S3 | `bucket`, `key_template` |
| `save_local` | Save locally | `filename`, `directory` |

### Control Effects
| Effect | Purpose | Key Parameters |
|--------|---------|----------------|
| `delay` | Add delay | `milliseconds`, `jitter` |
| `conditional` | If/then/else | `condition`, `then_effects` |
| `retry` | Retry effect | `effect_id`, `max_attempts` |

## Workflow Structure

```json
{
  "workflow_id": "scraper_id",
  "config": {
    "rate_limit": { "requests_per_second": 2 },
    "retry": { "max_attempts": 3 },
    "browser": { "headless": true }
  },
  "effects": [
    {
      "type": "navigate",
      "id": "goto_page",
      "params": { "url": "https://example.com" }
    }
  ],
  "schemas": {
    "Product": {
      "fields": [
        { "name": "id", "field_type": "string", "required": true }
      ]
    }
  }
}
```

## Template Variables

Use in string parameters:
- `{{ VARIABLE_NAME }}` - Environment variable
- `{{ date }}` - Current date (YYYY-MM-DD)
- `{{ timestamp }}` - Unix timestamp
- `{{ workflow_id }}` - Workflow ID
- `{{ execution_id }}` - Unique execution ID

## Extraction Field Types

```json
{
  "fields": {
    "text_field": {
      "type": "text",
      "selector": ".element"
    },
    "attr_field": {
      "type": "attribute",
      "selector": "[data-id]",
      "attribute": "data-id"
    },
    "transformed_field": {
      "type": "text",
      "selector": ".price",
      "transform": "parseFloat"
    }
  }
}
```

## Troubleshooting Checklist

### Chrome Not Found
```bash
# Set path explicitly
export PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium-browser

# Common paths:
# macOS: /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
# Ubuntu: /usr/bin/chromium-browser
# Windows: C:\Program Files\Google\Chrome\Application\chrome.exe
```

### S3 Upload Failed
```bash
# Test AWS credentials
aws s3 ls s3://your-bucket/

# Check environment
env | grep AWS

# Verify region
echo $AWS_REGION
```

### Rate Limiting
```json
// In workflow.json
"rate_limit": {
  "requests_per_second": 1.0,  // Reduce rate
  "burst": 3                    // Reduce burst
}
```

### Memory Issues
```bash
# Increase Node memory
NODE_OPTIONS=--max-old-space-size=4096 npm start

# Run headless
HEADLESS=true npm start
```

### Debug Mode
```bash
# Full debug output
LOG_LEVEL=debug HEADLESS=false npm run dev

# With Chrome DevTools
DEVTOOLS=true npm run dev

# Slow motion (see each action)
SLOW_MO=250 npm run dev
```

## Docker Quick Start

```bash
# Build image
docker build -t scraper .

# Run with env file
docker run --env-file .env scraper

# Run with individual vars
docker run \
  -e AWS_ACCESS_KEY_ID=key \
  -e AWS_SECRET_ACCESS_KEY=secret \
  -e S3_BUCKET=bucket \
  -e EXAMPLE_SHOP_USERNAME=user \
  -e EXAMPLE_SHOP_PASSWORD=pass \
  scraper

# Interactive debug
docker run -it --env-file .env scraper sh
```

## Airflow Integration

```python
from airflow.providers.docker.operators.docker import DockerOperator

scrape_task = DockerOperator(
    task_id='scrape_daily',
    image='scraper:latest',
    environment={
        'AWS_ACCESS_KEY_ID': '{{ var.value.aws_key }}',
        'S3_BUCKET': 'my-bucket'
    },
    docker_url='unix://var/run/docker.sock'
)
```

## File Locations

| Type | Location |
|------|----------|
| Source code | `src/` |
| Effects library | `lib/effects/` |
| Configuration | `workflow.json`, `.env` |
| Tests | `tests/` |
| Mock store | `tests/integration/mock-store/` |
| Build output | `dist/` |
| Test results | `tests/results/` |
| Screenshots | `tests/results/screenshots/` |

## Performance Tips

1. **Reduce wait times**: Use `networkidle0` instead of `networkidle2`
2. **Limit pages**: Set `max_pages` in pagination
3. **Headless mode**: Always use in production
4. **Optimize selectors**: Use IDs over complex selectors
5. **Batch operations**: Extract multiple fields at once

## Common Patterns

### Login Flow
```json
[
  { "type": "navigate", "id": "home" },
  { "type": "click", "id": "open_login" },
  { "type": "type", "id": "credentials" },
  { "type": "click", "id": "submit" },
  { "type": "wait_for_selector", "id": "verify_login" }
]
```

### Data Extraction
```json
[
  { "type": "navigate", "id": "data_page" },
  { "type": "wait_for_selector", "id": "wait_load" },
  { "type": "extract_all", "id": "extract" },
  { "type": "validate", "id": "validate" },
  { "type": "serialize_json", "id": "serialize" },
  { "type": "upload_s3", "id": "upload" }
]
```

### Error Recovery
```json
[
  { "type": "navigate", "id": "page" },
  { 
    "type": "conditional", 
    "id": "check_loaded",
    "params": {
      "condition": {
        "expression_type": "selector_exists",
        "field": ".content"
      },
      "then_effects": ["extract_data"],
      "else_effects": ["retry_navigation"]
    }
  }
]
```

## Support

- Documentation: See DOCUMENTATION.md
- Architecture: See ARCHITECTURE.md
- API Reference: See API_REFERENCE.md
- Testing Guide: See TESTING.md
- Issues: Check logs and screenshots first