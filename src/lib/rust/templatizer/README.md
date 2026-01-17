# Templatizer

Unified template generator for Spark, Puppeteer, and Superset projects.

## Overview

Templatizer consolidates three template generators into a single crate:

- **Spark**: Generate Python (PySpark) or Scala Spark applications
- **Puppeteer**: Generate Node.js web scrapers for Airflow/Kubernetes
- **Superset**: Generate Apache Superset dashboard configurations

## Installation

The crate is part of the Firestream workspace. Build it with:

```bash
cargo build -p templatizer
```

## CLI Usage

### Generate a Spark application

```bash
templatizer spark -n my-app -l python -o ./output
templatizer spark -n my-app -l scala -o ./output
```

### Generate a Puppeteer scraper

```bash
templatizer puppeteer -n my-scraper -t dom_scraper -o ./output
templatizer puppeteer -n my-scraper -t fn_scraper -o ./output
```

### Generate a Superset dashboard

```bash
templatizer superset -n my-dashboard -c config.yaml -o ./output
templatizer superset -n my-dashboard --zip -o ./dashboard.zip
```

### List available templates

```bash
templatizer list
templatizer list -f spark --detailed
```

## Library Usage

```rust
use templatizer::{TemplateEngine, TemplateType};
use tera::Context;

// Create an engine for Spark templates
let engine = TemplateEngine::new(TemplateType::Spark)?;

// List available templates
for template in engine.list_templates() {
    println!("Template: {}", template);
}

// Render a template
let mut context = Context::new();
context.insert("project_name", "my-spark-app");
let output = engine.render("main.py.tera", &context)?;
```

## Architecture

The crate uses `workspace-embed` to embed template files at compile time from
`src/templates/` in the repository root. At runtime, templates are extracted
to a temporary directory and processed with Tera.

### Module Structure

- `embedded`: Template embedding and extraction using rust-embed
- `engine`: Unified Tera template engine wrapper
- `config`: Shared configuration traits and types
- `error`: Error types for the crate
- `cli`: Command-line interface implementation
- `puppeteer`: Puppeteer-specific generator
- `spark`: Spark-specific generator
- `superset`: Superset-specific generator

## Template Locations

Templates are located in the repository at:

- `src/templates/spark/` - Spark templates (python and scala subdirectories)
- `src/templates/puppeteer/` - Puppeteer templates (dom_scraper and fn_scraper)
- `src/templates/superset/` - Superset templates (*.yaml.tera files)

## Development

### Adding new templates

1. Add template files to the appropriate directory under `src/templates/`
2. Rebuild the crate to embed the new templates
3. Templates will be available via the `TemplateEngine`

### Testing

```bash
cargo test -p templatizer
```

## License

MIT
