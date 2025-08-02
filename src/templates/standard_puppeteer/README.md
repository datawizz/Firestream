# example_shop Scraper

Auto-generated web scraper for example_shop using the effects-based Puppeteer framework.

## Overview

- **Workflow ID**: `ecommerce_product_scraper`
- **Version**: 1.0.0
- **Type**: dom-scraping
- **Airflow Task ID**: `scrape_products_daily`

## Features

- Production-ready scraper with no manual coding required
- Built-in retry logic with exponential backoff
- Rate limiting to respect server resources
- Automatic S3 upload for scraped data
- Docker support for containerized deployment
- Airflow DAG compatible
- Structured logging for monitoring
- Type-safe with full TypeScript support

## Quick Start

### Prerequisites

- Node.js 18+ 
- Chrome/Chromium browser
- AWS credentials (for S3 upload)
- Docker (optional)

### Installation

1. Install dependencies:
```bash
npm install
```

2. Copy environment configuration:
```bash
cp .env.example .env
```

3. Edit `.env` with your credentials and configuration

### Running the Scraper

#### Development Mode
```bash
npm run dev
```

#### Production Mode
```bash
npm run build
npm start
```

#### Using Make
```bash
make run
```

#### Using Docker
```bash
make docker-run
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `AWS_ACCESS_KEY_ID` | AWS access key for S3 | Yes |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key for S3 | Yes |
| `S3_BUCKET` | S3 bucket for data storage | Yes |
| `EXAMPLE_SHOP_USERNAME` | Login username | Yes |
| `EXAMPLE_SHOP_PASSWORD` | Login password | Yes |
| `LOG_LEVEL` | Logging level (debug/info/warn/error) | No |
| `HEADLESS` | Run browser in headless mode | No |

### Workflow Configuration

The scraper executes the following effects in order:

1. **navigate** (`goto_home`)
2. **click** (`open_login`)
   - Dependencies: goto_home
3. **type** (`enter_credentials`)
   - Dependencies: open_login
4. **click** (`submit_login`)
   - Dependencies: enter_credentials
5. **wait_for_selector** (`wait_dashboard`)
   - Dependencies: submit_login
6. **navigate** (`goto_products`)
   - Dependencies: wait_dashboard
7. **extract_all** (`extract_products`)
   - Dependencies: goto_products
8. **validate** (`validate_products`)
   - Dependencies: extract_products
9. **serialize_json** (`serialize_data`)
   - Dependencies: validate_products
10. **upload_s3** (`upload_results`)
   - Dependencies: serialize_data


## Data Output

### Schema

The scraper extracts the following fields:

**Product**:
- `id` (string): Product ID
- `name` (string): Product name
- `price` (number): Product price
- `inStock` (boolean): Stock availability



### Output Location

- **S3**: `s3://my-data-lake/scraped-data/products/<date>/<workflow_id>-<timestamp>.json`
- **Local** (development): `./output/<workflow_id>/<timestamp>.json`

## Deployment

### Airflow

Deploy to your Airflow DAGs directory:

```bash
make deploy-airflow AIRFLOW_DAGS_DIR=/path/to/dags
```

Or manually:

```python
from airflow import DAG
from airflow.providers.docker.operators.docker import DockerOperator
from datetime import datetime, timedelta

default_args = {
    'owner': 'data-team',
    'retries': 1,
    'retry_delay': timedelta(minutes=5),
}

with DAG(
    'ecommerce_product_scraper',
    default_args=default_args,
    description='example_shop scraper',
    schedule='@daily',
    start_date=datetime(2024, 1, 1),
    catchup=False,
) as dag:
    
    scrape_task = DockerOperator(
        task_id='scrape_products_daily',
        image='local/example_shop-scraper:latest',
        environment={
            'AWS_ACCESS_KEY_ID': '{{ var.value.aws_access_key }}',
            'AWS_SECRET_ACCESS_KEY': '{{ var.value.aws_secret_key }}',
            'S3_BUCKET': 'my-data-lake',
            # Add other environment variables
        },
        docker_url='unix://var/run/docker.sock',
        network_mode='bridge',
    )
```

### Kubernetes

Deploy as a CronJob:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: ecommerce_product_scraper
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: scraper
            image: local/example_shop-scraper:latest
            envFrom:
            - secretRef:
                name: example_shop-scraper-secrets
          restartPolicy: OnFailure
```

## Monitoring

### Logs

The scraper outputs structured JSON logs suitable for ingestion into logging systems:

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

### Metrics

Key metrics to monitor:
- Execution time per effect
- Number of items extracted
- Error rate
- S3 upload success rate

## Troubleshooting

### Common Issues

1. **Chrome/Chromium not found**
   - Ensure `PUPPETEER_EXECUTABLE_PATH` points to your Chrome installation
   - In Docker, this is handled automatically

2. **S3 upload fails**
   - Check AWS credentials in `.env`
   - Verify S3 bucket exists and has proper permissions
   - Check AWS region configuration

3. **Rate limiting errors**
   - Adjust `RATE_LIMIT_RPS` in `.env`
   - Increase `RETRY_BACKOFF_MS` for aggressive retries

4. **Memory issues**
   - Set `NODE_OPTIONS=--max-old-space-size=4096` for more memory
   - Enable headless mode: `HEADLESS=true`

### Debug Mode

Run with debug logging:
```bash
LOG_LEVEL=debug npm run dev
```

Run with browser visible:
```bash
HEADLESS=false npm run dev
```

## Development

### Running Tests
```bash
npm test
```

### Type Checking
```bash
npm run typecheck
```

### Linting
```bash
npm run lint
```

## License

This scraper was generated by templatizer-puppeteer and is provided as-is for use with example_shop.

---

Generated by templatizer-puppeteer v1.0.0