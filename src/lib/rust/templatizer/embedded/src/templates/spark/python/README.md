# Spark Python Boilerplate Templates

This directory contains Tera templates for generating a complete PySpark application that can be deployed on Kubernetes using the Spark Operator.

## Template Structure

```
spark_python_boilerplate/
├── src/
│   ├── __init__.py.tera           # Package initialization
│   ├── main.py.tera               # Main PySpark application
│   ├── config.py.tera             # Configuration management
│   └── utils.py.tera              # Utility functions
├── tests/
│   ├── __init__.py.tera           # Test package init
│   ├── test_main.py.tera          # Unit and integration tests
│   └── conftest.py.tera           # Pytest configuration
├── resources/
│   ├── config.json.tera           # Default configuration
│   └── log4j.properties.tera      # Spark logging config
├── k8s/
│   ├── spark-application.yaml.tera # SparkApplication resource
│   ├── rbac.yaml.tera             # RBAC configuration
│   └── configmap.yaml.tera        # ConfigMap for configs
├── requirements.txt.tera          # Python dependencies
├── setup.py.tera                  # Package setup
├── pyproject.toml.tera           # Modern Python packaging
├── pytest.ini.tera               # Pytest configuration
├── Dockerfile.tera               # Docker image definition
├── Makefile.tera                 # Build automation
├── README.md.tera                # Project documentation
├── .gitignore.tera              # Git ignore patterns
├── .dockerignore.tera           # Docker ignore patterns
└── .pre-commit-config.yaml.tera # Pre-commit hooks
```

## Using the Templates

### From Rust Code

```rust
use tera::{Tera, Context};
use serde_json::json;

// Load the templates
let mut tera = Tera::new("src/templates/spark_python_boilerplate/**/*.tera")?;

// Create the context
let mut context = Context::new();
context.insert("app_name", "My PySpark App");
context.insert("organization", "com.mycompany");
context.insert("version", "1.0.0");
context.insert("package_name", "my_spark_app");
context.insert("docker_registry", "myregistry.io");
// ... add more variables as needed

// Render a template
let main_py = tera.render("src/main.py.tera", &context)?;

// Or use JSON context
let context = Context::from_serialize(json!({
    "app_name": "My PySpark App",
    "organization": "com.mycompany",
    "version": "1.0.0",
    // ... more fields
}))?;
```

### Template Variables

See `TEMPLATE_VARIABLES.md` for a complete list of available template variables.

### Example Context

See `example_context.json` for a complete example of a context that can be used to render the templates.

## Features

The templates support:

- **Python Best Practices**: Modern Python packaging with pyproject.toml, type hints, proper testing
- **Configurable Dependencies**: Enable/disable AWS S3, Delta Lake support
- **Flexible Data Processing**: Configure input/output formats, transformations, aggregations
- **Configuration Management**: Support for JSON/YAML/TOML configs with environment overrides
- **Kubernetes Deployment**: Complete K8s resources with RBAC, ConfigMaps, volumes
- **Docker Support**: Optimized Docker builds with proper Python setup
- **Testing**: Comprehensive pytest setup with fixtures, markers, and coverage
- **Code Quality**: Pre-commit hooks, black formatting, flake8 linting, mypy type checking
- **Build Automation**: Makefile with common tasks
- **Documentation**: Auto-generated README with usage instructions

## Differences from Scala Template

While maintaining configuration compatibility with the Scala template, the Python template has some differences:

1. **Package Management**: Uses pip/setuptools instead of SBT
2. **Testing Framework**: Uses pytest instead of ScalaTest
3. **Code Organization**: Python module structure instead of Java packages
4. **Docker Base Image**: Uses `apache/spark-py` instead of regular Spark image
5. **Application Type**: Sets `type: Python` in SparkApplication
6. **Entry Point**: Uses Python module path instead of JAR file

## Customization

### Adding New Templates

1. Create a new `.tera` file in the appropriate directory
2. Use Tera syntax for templating
3. Follow Python naming conventions (snake_case for files and functions)

### Extending Existing Templates

Templates use conditional blocks to include/exclude features:

```tera
{% if s3_enabled %}
# AWS dependencies
boto3>=1.26.0
{% endif %}
```

### Custom Transformations

Add transformations using PySpark syntax:

```json
"transformations": [
  {
    "method": "filter",
    "args": ["F.col('status') == 'active'"]
  },
  {
    "method": "withColumn",
    "args": ["'year'", "F.year(F.col('date'))"]
  }
]
```

## Best Practices

1. **Use Type Hints**: Templates include type hints for better IDE support
2. **Virtual Environments**: Always use virtual environments for development
3. **Configuration**: Use dataclasses for configuration management
4. **Testing**: Write comprehensive tests with proper fixtures
5. **Code Quality**: Use pre-commit hooks to maintain code quality
6. **Documentation**: Generate comprehensive documentation with examples

## Integration with CI/CD

The generated project includes:

- Dockerfile for containerization
- Makefile for common tasks
- pytest configuration for testing
- Pre-commit hooks for code quality
- K8s resources for deployment

This enables easy integration with CI/CD pipelines for automated building, testing, and deployment.

## Python-Specific Features

1. **Interactive Development**: 
   - `make shell` - Start IPython with Spark pre-configured
   - `make notebook` - Start Jupyter notebook

2. **Code Quality Tools**:
   - Black for formatting
   - isort for import sorting
   - flake8 for linting
   - mypy for type checking

3. **Testing Features**:
   - pytest with fixtures
   - Coverage reporting
   - Integration test support
   - Mock S3 support for testing

4. **Packaging**:
   - setuptools and wheel support
   - Console script entry points
   - Modern pyproject.toml configuration

## Configuration Compatibility

The Python template maintains compatibility with the Scala template configuration:

- Same Kubernetes resource structure
- Same environment variable patterns
- Same configuration field naming
- Same Docker registry approach
- Same monitoring and logging setup

This allows using the same configuration management system for both Scala and Python Spark applications.
