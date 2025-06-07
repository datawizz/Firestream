# Spark Templatizer

Generate production-ready Apache Spark applications for Kubernetes deployment using Tera templates.

## Overview

Spark Templatizer provides comprehensive templates for creating Spark applications in both Scala and Python. The generated applications are designed to run on Kubernetes using the Spark Operator and include:

- Complete project structure
- Build configurations (SBT for Scala, setuptools for Python)
- Docker support with optimized images
- Kubernetes deployment manifests
- Comprehensive testing setup
- CI/CD ready configurations

## Features

- **Multi-language Support**: Generate Scala or Python Spark applications
- **Production Ready**: Includes monitoring, logging, error handling
- **Cloud Native**: Built for Kubernetes with Spark Operator
- **Configurable**: 100+ template variables for customization
- **Best Practices**: Follows Spark and language-specific best practices
- **Testing**: Comprehensive test templates included
- **Documentation**: Auto-generated README and documentation

## Installation

### As a CLI Tool

```bash
cargo install --path .
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
spark_templatizer = "0.1.0"
```

## Usage

### CLI Usage

```bash
# Show help
spark-templatizer --help

# Generate a Scala Spark application
cargo run generate \
  --template scala \
  --config /workspace/src/lib/rust/spark_templatizer/src/templates/spark_scala_boilerplate/example_context.json \
  --output ./_out/my-spark-app

# Generate a Python Spark application
cargo run generate \
  --template python \
  --config src/templates/spark_python_boilerplate/example_context.json \
  --output ./_out/my-pyspark-app

# Show example configuration
spark-templatizer example --template scala > my-config.json

# List available template variables
spark-templatizer list-vars --template python
```

### Library Usage

```rust
use spark_templatizer::{SparkTemplatizer, TemplateType, SparkAppConfig};
use std::path::Path;

// Using the config builder
let config = SparkAppConfig::new("My Spark App", "1.0.0")
    .organization("com.example")
    .docker_registry("myregistry.io")
    .enable_s3()
    .enable_delta()
    .namespace("spark-apps")
    .build()?;

// Generate Scala application
let mut scala_gen = SparkTemplatizer::new(TemplateType::Scala)?;
scala_gen.render_to_directory(config.clone(), Path::new("./scala-app"))?;

// Generate Python application
let mut python_gen = SparkTemplatizer::new(TemplateType::Python)?;
python_gen.render_to_directory(config, Path::new("./python-app"))?;
```

## Configuration

Create a JSON configuration file with your application settings:

```json
{
  "app_name": "Sales Analytics",
  "organization": "com.example",
  "version": "1.0.0",
  "description": "Spark application for sales data analytics",
  "docker_registry": "myregistry.io",

  "s3_enabled": true,
  "delta_enabled": true,
  "config_enabled": true,

  "driver_memory": "4g",
  "executor_memory": "8g",
  "executor_instances": 5,

  "config_fields": [
    {
      "name": "inputPath",
      "type": "String",
      "default_value": "s3a://data-lake/sales/raw",
      "env_var": "INPUT_PATH"
    }
  ]
}
```

See the example configurations for complete examples:
- Scala: `src/templates/spark_scala_boilerplate/example_context.json`
- Python: `src/templates/spark_python_boilerplate/example_context.json`

## Generated Project Structure

### Scala Project
```
my-spark-app/
├── build.sbt
├── project/
│   ├── build.properties
│   └── plugins.sbt
├── src/
│   ├── main/scala/
│   └── test/scala/
├── Dockerfile
├── k8s/
│   ├── spark-application.yaml
│   ├── rbac.yaml
│   └── configmap.yaml
└── Makefile
```

### Python Project
```
my-pyspark-app/
├── src/
│   └── my_app/
│       ├── __init__.py
│       ├── main.py
│       ├── config.py
│       └── utils.py
├── tests/
├── requirements.txt
├── setup.py
├── pyproject.toml
├── Dockerfile
├── k8s/
└── Makefile
```

## Building and Deploying Generated Applications

### Scala Application

```bash
cd my-spark-app

# Build
sbt assembly

# Test
sbt test

# Docker
make docker-build
make docker-push

# Deploy to Kubernetes
make k8s-deploy
```

### Python Application

```bash
cd my-pyspark-app

# Setup
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Test
pytest

# Docker
make docker-build
make docker-push

# Deploy to Kubernetes
make k8s-deploy
```

## Template Variables

Both templates support extensive customization through template variables:

- **Core**: app_name, version, organization, description
- **Features**: s3_enabled, delta_enabled, streaming_enabled
- **Resources**: driver_memory, executor_memory, executor_instances
- **Docker/K8s**: docker_registry, namespace, service_account
- **Configuration**: config_fields, spark_configs, environment_vars

See `TEMPLATE_VARIABLES.md` in each template directory for complete documentation.

## Development

### Project Structure

```
spark_templatizer/
├── src/
│   ├── templates/
│   │   ├── spark_scala_boilerplate/
│   │   └── spark_python_boilerplate/
│   ├── lib.rs
│   ├── main.rs
│   └── example.rs
├── Cargo.toml
└── README.md
```

### Running Tests

```bash
cargo test
```

### Adding New Templates

1. Create a new directory under `src/templates/`
2. Add `.tera` template files
3. Create `TEMPLATE_VARIABLES.md` documentation
4. Add example configuration
5. Update the `TemplateType` enum

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

Apache License 2.0

## Acknowledgments

- [Tera](https://tera.netlify.app/) - Template engine
- [Kubeflow Spark Operator](https://github.com/kubeflow/spark-operator) - Kubernetes operator for Spark
- Apache Spark community

## Support

For issues and questions:
- Open an issue on GitHub
- Check the template documentation
- Review example configurations
