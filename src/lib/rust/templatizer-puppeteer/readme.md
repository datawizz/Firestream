# Poo Scraper Templatizer

Generate production-ready web scrapers for Kubernetes/Airflow deployment using Tera templates.

## Overview

This tool generates TypeScript-based web scrapers designed to run as single-execution pods in Kubernetes, scheduled by Apache Airflow. It supports two workflows:

1. **DOM Scraping**: Login → Navigate → Extract from DOM → Save to S3
2. **API Scraping**: Login → Extract Auth → Use Auth for APIs → Save to S3

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Generate a Scraper

```bash
# Using the example configuration
cargo run generate -c example_api_scraper.json -o ./my-scraper

# Using your own configuration
pooscraper-templatizer generate -c config.json -o ./output-dir
```

### Show Example Configuration

```bash
pooscraper-templatizer example
```

### List Template Variables

```bash
pooscraper-templatizer list-vars
```

## Configuration

Create a JSON configuration file with the required variables:

```json
{
  "site_name": "example_shop",
  "site_url": "https://shop.example.com",
  "workflow_type": "dom-scraping",

  "auth_type": "form",
  "login_username_selector": "input[name='email']",
  "login_password_selector": "input[name='password']",
  "login_submit_selector": "button[type='submit']",
  "login_success_selector": ".dashboard",
  "login_error_selector": ".error",

  "s3_bucket": "my-data-lake",
  "s3_region": "us-east-1",
  "output_format": "parquet",

  "navigation_steps": [
    {
      "name": "products",
      "url": "/products",
      "wait_type": "selector",
      "wait_value": ".product-list",
      "screenshot": true
    }
  ]
}
```

## Generated Structure

The tool generates a complete TypeScript project:

```
my-scraper/
├── package.json          # Node.js dependencies
├── tsconfig.json         # TypeScript configuration
├── Dockerfile            # Container configuration
├── index.ts              # Entry point
├── lib/                  # Framework code (DO NOT MODIFY)
│   ├── core/            # Core utilities
│   ├── workflows/       # Workflow implementations
│   └── utils/           # Helper utilities
└── src/                  # Site-specific implementations
    ├── config.ts        # Configuration
    ├── data-extractor.ts # DOM scraping logic
    └── api-client.ts    # API scraping logic
```

## Workflow Selection

### DOM Scraping Workflow

Use this when:
- Data is rendered in HTML
- No API is available
- Data requires navigation through multiple pages

Configuration focus:
- CSS selectors for login
- Navigation steps
- DOM extraction logic

### API Scraping Workflow

Use this when:
- Site has a JSON API
- More efficient than DOM scraping
- Need to make multiple API calls with authentication

Configuration focus:
- Auth token extraction
- API endpoints
- Response processing

## Development Workflow

1. **Generate the scraper**:
   ```bash
   pooscraper-templatizer generate -c config.json -o ./my-scraper
   ```

2. **Implement site-specific logic**:
   - For DOM: Edit `src/data-extractor.ts`
   - For API: Edit `src/api-client.ts`

3. **Test locally**:
   ```bash
   cd my-scraper
   npm install
   npm run dev
   ```

4. **Build Docker image**:
   ```bash
   docker build -t my-scraper:latest .
   ```

5. **Deploy to Kubernetes**:
   ```bash
   kubectl apply -f k8s/
   ```

## Library Usage

You can also use this as a library in your Rust projects:

```rust
use pooscraper_templatizer::{PooScraperTemplatizer, ScraperConfig, WorkflowType};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = ScraperConfig::new("my_shop", "https://shop.example.com", WorkflowType::DomScraping)
        .s3_bucket("my-bucket")
        .docker_registry("myregistry.io")
        .add_navigation_step("products", "/products", "selector", ".products", true)
        .build()?;

    // Generate templates
    let mut generator = PooScraperTemplatizer::new()?;
    generator.render_to_directory(config, Path::new("./output"))?;

    Ok(())
}
```

## Testing

Run the test suite:

```bash
cargo test
```

This will test:
- Template rendering for both workflows
- Configuration building
- Variable substitution

## License

MIT
