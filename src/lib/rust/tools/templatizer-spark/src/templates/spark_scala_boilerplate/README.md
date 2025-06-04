# Spark Scala Boilerplate Templates

This directory contains Tera templates for generating a complete Spark Scala application that can be deployed on Kubernetes using the Spark Operator.

## Template Structure

```
spark_scala_boilerplate/
├── build.sbt.tera                  # SBT build configuration
├── project/
│   ├── build.properties.tera       # SBT version configuration
│   └── plugins.sbt.tera            # SBT plugins
├── src/
│   ├── main/
│   │   ├── scala/
│   │   │   └── SparkApp.scala.tera # Main Spark application
│   │   └── resources/
│   │       ├── application.conf.tera # Application configuration
│   │       └── log4j.properties.tera # Logging configuration
│   └── test/
│       └── scala/
│           └── SparkAppTest.scala.tera # Unit tests
├── Dockerfile.tera                 # Docker image definition
├── k8s/
│   ├── spark-application.yaml.tera # SparkApplication resource
│   ├── rbac.yaml.tera             # RBAC configuration
│   └── configmap.yaml.tera        # ConfigMap for configs
├── Makefile.tera                  # Build automation
├── README.md.tera                 # Project documentation
├── .gitignore.tera               # Git ignore patterns
└── .dockerignore.tera            # Docker ignore patterns
```

## Using the Templates

### From Rust Code

```rust
use tera::{Tera, Context};
use serde_json::json;

// Load the templates
let mut tera = Tera::new("src/templates/spark_scala_boilerplate/**/*.tera")?;

// Create the context
let mut context = Context::new();
context.insert("app_name", "My Spark App");
context.insert("organization", "com.mycompany");
context.insert("version", "1.0.0");
context.insert("package_name", "com.mycompany.spark");
context.insert("docker_registry", "myregistry.io");
// ... add more variables as needed

// Render a template
let build_sbt = tera.render("build.sbt.tera", &context)?;

// Or use JSON context
let context = Context::from_serialize(json!({
    "app_name": "My Spark App",
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

- **Configurable Dependencies**: Enable/disable Spark Streaming, MLlib, S3, Delta Lake
- **Flexible Data Processing**: Configure input/output formats, transformations, aggregations
- **Kubernetes Deployment**: Complete K8s resources with RBAC, ConfigMaps, volumes
- **Docker Support**: Multi-stage builds, custom base images, additional packages
- **Testing**: Unit and integration test templates
- **Build Automation**: Makefile with common tasks
- **Configuration Management**: Typesafe Config support with environment overrides
- **Monitoring**: Prometheus metrics export support
- **Documentation**: Auto-generated README with usage instructions

## Customization

### Adding New Templates

1. Create a new `.tera` file in the appropriate directory
2. Use Tera syntax for templating:
   - `{{ variable }}` for expressions
   - `{% if condition %}...{% endif %}` for conditionals
   - `{% for item in items %}...{% endfor %}` for loops

### Extending Existing Templates

Templates use conditional blocks to include/exclude features:

```tera
{% if s3_enabled %}
// AWS SDK for S3
"org.apache.hadoop" % "hadoop-aws" % hadoopVersion % "provided",
{% endif %}
```

### Custom Transformations

Add transformations to the context:

```json
"transformations": [
  {
    "method": "filter",
    "args": ["col(\"status\") === \"active\""]
  },
  {
    "method": "withColumn",
    "args": ["\"year\"", "year(col(\"date\"))"]
  }
]
```

## Best Practices

1. **Use Defaults**: Templates include sensible defaults via the `default` filter
2. **Conditional Features**: Use feature flags to keep generated code clean
3. **Environment Variables**: Support configuration via environment variables
4. **Version Compatibility**: Ensure Spark, Scala, and dependency versions are compatible
5. **Documentation**: Generate comprehensive documentation with the templates

## Integration with CI/CD

The generated project includes:

- Dockerfile for containerization
- Makefile for common tasks
- K8s resources for deployment
- Test templates for quality assurance

This enables easy integration with CI/CD pipelines for automated building, testing, and deployment.
