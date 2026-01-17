# E-commerce Web Scraper Documentation

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Getting Started](#getting-started)
4. [Configuration](#configuration)
5. [Usage](#usage)
6. [Effects System](#effects-system)
7. [Testing](#testing)
8. [Deployment](#deployment)
9. [Monitoring & Logging](#monitoring--logging)
10. [Troubleshooting](#troubleshooting)
11. [Contributing](#contributing)

## Project Overview

The E-commerce Web Scraper is an automated, production-ready web scraping solution built with TypeScript and Puppeteer. It uses a declarative, effects-based architecture to extract product data from e-commerce websites.

### Key Features

- **Effects-based Architecture**: Modular, reusable scraping components
- **Declarative Workflows**: JSON-based workflow definitions
- **Production Ready**: Rate limiting, retry logic, and error handling
- **Cloud Storage**: Automatic S3 upload for scraped data
- **Container Support**: Docker and Kubernetes ready
- **Airflow Integration**: Native support for Apache Airflow
- **Comprehensive Testing**: Integration tests with mock store
- **Type Safety**: Full TypeScript support with strict typing

### Use Cases

- Daily product catalog scraping
- Price monitoring
- Inventory tracking
- Competitive analysis
- Data lake population

## Architecture

The scraper uses an effects-based architecture where each scraping action is encapsulated as an "effect". Effects can be composed into workflows that are executed in dependency order.

### Core Components

1. **Effects**: Individual scraping actions (navigate, click, extract, etc.)
2. **Workflow Engine**: Executes effects in dependency order
3. **Runtime Context**: Manages browser, page, and shared data
4. **Rate Limiter**: Controls request frequency
5. **Retry Manager**: Handles transient failures
6. **Data Validators**: Ensures data quality

### Workflow Execution Flow

```
Start → Load Workflow → Initialize Browser → Execute Effects → Validate Data → Upload to S3 → Complete
```

For detailed architecture information, see [ARCHITECTURE.md](./ARCHITECTURE.md).

## Getting Started

### Prerequisites

- Node.js 18+
- Chrome/Chromium browser
- AWS account (for S3 storage)
- Docker (optional)

### Installation

1. Clone the repository:
```bash
cd /workspace/src/dags/ecommerce
```

2. Install dependencies:
```bash
npm install
```

3. Copy environment configuration:
```bash
cp .env.example .env
```

4. Edit `.env` with your credentials:
```bash
# Required variables
AWS_ACCESS_KEY_ID=your_key_here
AWS_SECRET_ACCESS_KEY=your_secret_here
S3_BUCKET=your-bucket-name
EXAMPLE_SHOP_USERNAME=your_username
EXAMPLE_SHOP_PASSWORD=your_password
```

### Quick Start

```bash
# Development mode with hot reload
npm run dev

# Production build and run
npm run build
npm start

# Using Make
make run

# Using Docker
make docker-run
```

## Configuration

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `AWS_ACCESS_KEY_ID` | AWS access key for S3 | Yes | - |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key for S3 | Yes | - |
| `S3_BUCKET` | S3 bucket name | Yes | - |
| `EXAMPLE_SHOP_USERNAME` | Login username | Yes | - |
| `EXAMPLE_SHOP_PASSWORD` | Login password | Yes | - |
| `LOG_LEVEL` | Logging level | No | info |
| `HEADLESS` | Run browser headless | No | true |
| `PUPPETEER_EXECUTABLE_PATH` | Chrome path | No | Auto-detect |

### Workflow Configuration

The workflow is defined in `workflow.json`. Key configuration sections:

```json
{
  "config": {
    "rate_limit": {
      "requests_per_second": 2.0,
      "burst": 5
    },
    "retry": {
      "max_attempts": 3,
      "backoff_ms": 1000,
      "exponential": true
    },
    "browser": {
      "headless": true,
      "viewport": {
        "width": 1920,
        "height": 1080
      }
    }
  }
}
```

## Usage

### Basic Usage

```typescript
// The scraper runs automatically when started
// It executes the workflow defined in workflow.json
npm start
```

### Workflow Structure

The workflow consists of effects executed in order:

1. **Navigate to homepage**
2. **Click login button**
3. **Enter credentials**
4. **Submit login form**
5. **Wait for dashboard**
6. **Navigate to products**
7. **Extract product data**
8. **Validate extracted data**
9. **Serialize to JSON**
10. **Upload to S3**

### Data Output

Extracted data follows this schema:

```typescript
interface Product {
  id: string;          // Product ID
  name: string;        // Product name
  price: number;       // Product price
  inStock: boolean;    // Availability status
}
```

Output location: `s3://your-bucket/scraped-data/products/YYYY-MM-DD/workflow_id-timestamp.json`

## Effects System

### Available Effect Types

#### DOM Manipulation
- `navigate`: Navigate to URL
- `click`: Click element
- `type`: Type into fields
- `wait_for_selector`: Wait for element
- `extract_all`: Extract multiple items
- `extract_text`: Extract text content
- `screenshot`: Take screenshot

#### Data Processing
- `validate`: Validate against schema
- `serialize_json`: Convert to JSON
- `transform`: Transform data

#### Storage
- `upload_s3`: Upload to S3
- `save_local`: Save locally

#### Control Flow
- `delay`: Add delay
- `conditional`: Conditional execution
- `retry`: Retry failed effects

### Creating Custom Effects

```typescript
export class CustomEffect extends BaseEffect<ParamsType, InputType, OutputType> {
  async execute(input: InputType, context: ExecutionContext): Promise<OutputType> {
    // Implementation
    return output;
  }
}
```

For complete API reference, see [API_REFERENCE.md](./API_REFERENCE.md).

## Testing

### Running Tests

```bash
# Run all tests
npm run test:integration

# Run with visible browser
npm run test:integration:debug

# Run specific test
node tests/runTests.js --test=authentication
```

### Mock Store

The project includes a React-based mock store for testing:

```bash
# Start mock store
cd tests/integration/mock-store && npm start

# Access at http://localhost:3000
```

### Test Scenarios

1. **Authentication Test**: Login flow validation
2. **Product Extraction Test**: Data extraction verification
3. **Full Workflow Test**: End-to-end workflow execution

For detailed testing documentation, see [TESTING.md](./TESTING.md).

## Deployment

### Docker Deployment

```bash
# Build image
make docker

# Run container
docker run --env-file .env local/example_shop-scraper:latest
```

### Airflow Integration

```python
from airflow.providers.docker.operators.docker import DockerOperator

scrape_task = DockerOperator(
    task_id='scrape_products_daily',
    image='local/example_shop-scraper:latest',
    environment={
        'AWS_ACCESS_KEY_ID': '{{ var.value.aws_access_key }}',
        'AWS_SECRET_ACCESS_KEY': '{{ var.value.aws_secret_key }}',
        'S3_BUCKET': 'my-data-lake'
    }
)
```

### Kubernetes Deployment

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: ecommerce-scraper
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: scraper
            image: local/example_shop-scraper:latest
            envFrom:
            - secretRef:
                name: scraper-secrets
```

## Monitoring & Logging

### Structured Logging

The scraper outputs structured JSON logs:

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "message": "Effect completed: extract_products",
  "context": {
    "workflowId": "ecommerce_product_scraper",
    "executionId": "abc-123",
    "effectId": "extract_products"
  }
}
```

### Key Metrics

- Execution time per effect
- Number of items extracted
- Error rate by effect type
- S3 upload success rate

### Airflow Integration

The scraper provides Airflow-compatible logging:

```javascript
logger.airflowLog('complete', {
  workflow_id: workflow.workflow_id,
  duration_ms: duration,
  status: result.status,
  data_count: Object.keys(result.data).length
});
```

## Troubleshooting

### Common Issues

#### Chrome/Chromium Not Found

```bash
# Set executable path
export PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium-browser

# Or in .env
PUPPETEER_EXECUTABLE_PATH=/Applications/Google Chrome.app/Contents/MacOS/Google Chrome
```

#### S3 Upload Failures

1. Verify AWS credentials:
```bash
aws s3 ls s3://your-bucket/
```

2. Check IAM permissions for `s3:PutObject`

3. Verify bucket region matches configuration

#### Rate Limiting Errors

Adjust rate limiting in workflow.json:
```json
{
  "rate_limit": {
    "requests_per_second": 1.0,
    "burst": 3
  }
}
```

#### Memory Issues

```bash
# Increase Node.js memory
NODE_OPTIONS=--max-old-space-size=4096 npm start

# Enable headless mode
HEADLESS=true npm start
```

### Debug Mode

```bash
# Enable debug logging
LOG_LEVEL=debug npm run dev

# Run with visible browser
HEADLESS=false npm run dev

# Enable DevTools
DEVTOOLS=true npm run dev
```

### Getting Help

1. Check existing issues
2. Review test output and screenshots
3. Enable debug logging
4. Run with visible browser to see what's happening

## Contributing

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Style

- Use TypeScript for all new code
- Follow existing patterns in the codebase
- Add types for all parameters and return values
- Write descriptive commit messages

### Testing Requirements

- Add integration tests for new effects
- Update mock store if needed
- Ensure backward compatibility
- Document any breaking changes

### Documentation

- Update relevant documentation
- Add JSDoc comments for new functions
- Include examples for new features
- Update API reference if needed