# Spark Python Boilerplate Template Variables

This document describes all available variables that can be used with the Spark Python boilerplate templates.

## Core Variables

### Application Metadata
- `app_name` (string, required): Name of the Spark application
- `organization` (string, required): Organization name for the project
- `version` (string, required): Application version
- `description` (string): Application description
- `package_name` (string): Python package name (defaults to app_name lowercased with underscores)
- `main_class_name` (string, default: "SparkApp"): Main class name
- `author` (string): Author name
- `author_email` (string): Author email
- `project_url` (string): Project URL
- `contact_info` (string): Contact information

### Python Configuration
- `python_version` (string, default: "3.10"): Python version
- `python_major_version` (string, default: "3"): Python major version
- `pyspark_version` (string, default: "3.5.1"): PySpark version
- `development_status` (string, default: "3 - Alpha"): PyPI development status

### Features Flags
- `s3_enabled` (boolean): Include S3/AWS dependencies
- `delta_enabled` (boolean): Include Delta Lake support
- `config_enabled` (boolean): Include configuration management
- `checkpoint_enabled` (boolean): Enable checkpoint directory support
- `cache_input` (boolean): Cache input DataFrame
- `include_utils` (boolean): Include utility functions module
- `include_integration_test` (boolean): Include integration tests
- `monitoring_enabled` (boolean): Enable Prometheus monitoring
- `file_logging` (boolean): Enable file-based logging
- `logging_config` (boolean): Include logging configuration
- `include_dev_dependencies` (boolean): Include development dependencies
- `use_pre_commit` (boolean): Include pre-commit configuration

### Data Processing
- `input_format` (string): Input data format (json, parquet, csv, delta)
- `output_format` (string): Output data format (json, parquet, csv, delta)
- `output_mode` (string, default: "overwrite"): Output write mode
- `compression` (string, default: "snappy"): Compression codec
- `transformations` (array): List of transformations to apply
  - `method` (string): Transformation method name
  - `args` (array): Method arguments (as Python expressions)
- `custom_transformation_code` (string): Custom transformation code to inject
- `aggregations` (array): List of aggregations
  - `function` (string): Aggregation function
  - `column` (string): Column to aggregate
  - `alias` (string): Output column alias
- `group_by_columns` (array): Columns for groupBy operation

### Configuration Management
- `config_format` (string, default: "json"): Configuration file format (json, yaml, toml)
- `config_fields` (array): Application configuration fields
  - `name` (string): Field name
  - `python_type` (string): Python type (str, int, float, bool)
  - `python_default` (string): Default value as Python expression
  - `default_value` (string): Default value
  - `runtime_value` (string): Runtime value for ConfigMap
  - `env_var` (string): Environment variable name
  - `arg_index` (number): Command-line argument index
  - `description` (string): Field description
  - `test_value` (string): Value for testing
  - `test_env_value` (string): Test environment value
  - `test_arg_value` (string): Test argument value

### Spark Configuration
- `spark_version` (string, default: "3.5.1"): Apache Spark version
- `spark_configs` (array): Spark configuration settings
  - `key` (string): Configuration key
  - `value` (string): Configuration value
  - `env_var` (string): Environment variable override
- `spark_conf` (map): Spark configuration map for K8s
- `hadoop_conf` (map): Hadoop configuration map

### Docker Configuration
- `docker_registry` (string, required): Docker registry URL
- `spark_base_image` (string, default: "apache/spark-py:v3.5.1"): Base Spark Docker image
- `image_pull_policy` (string, default: "Always"): Image pull policy
- `image_pull_secrets` (array): Image pull secrets
- `apt_packages` (array): Additional apt packages to install
- `additional_files` (array): Additional files to copy
  - `source` (string): Source path
  - `destination` (string): Destination path
- `config_files` (array): Configuration files to copy
- `environment_vars` (array): Environment variables

### Python Dependencies
- `delta_spark_version` (string, default: "2.4.0"): Delta Spark version
- `additional_requirements` (array): Additional pip requirements
- `extras_require` (map): Extra requirements for setup.py
- `package_data_files` (array): Package data files to include

### Kubernetes Configuration
- `namespace` (string, default: "spark-apps"): Kubernetes namespace
- `service_account` (string, default: "spark"): Service account name
- `labels` (map): Additional labels
- `arguments` (array): Application arguments
- `py_files` (array): Python files to distribute
- `packages` (array): Maven packages to include
- `repositories` (array): Maven repositories

### Testing
- `test_sample_data` (array): Sample data for tests (as Python tuples)
- `test_columns` (array): Test DataFrame columns
- `test_schema` (array): Test DataFrame schema
  - Column name and PySpark type pairs
- `expected_output_schema` (array): Expected output schema
- `additional_tests` (array): Additional test cases
  - `name` (string): Test name
  - `description` (string): Test description
  - `body` (string): Test body
- `additional_pytest_markers` (array): Additional pytest markers
  - `name` (string): Marker name
  - `description` (string): Marker description
- `custom_fixtures` (array): Custom pytest fixtures
  - `name` (string): Fixture name
  - `description` (string): Fixture description
  - `dependencies` (array): Fixture dependencies
  - `body` (string): Fixture body

### Code Quality
- `max_line_length` (number, default: 100): Maximum line length
- `additional_pre_commit_hooks` (array): Additional pre-commit hooks
  - `repo` (string): Repository URL
  - `rev` (string): Revision
  - `id` (string): Hook ID
  - `args` (array): Hook arguments
  - `additional_dependencies` (array): Additional dependencies

### Logging
- `log_level` (string, default: "INFO"): Log level
- `log_file_path` (string): Log file path
- `log_file_size` (string, default: "100MB"): Log file size
- `log_file_size_bytes` (number): Log file size in bytes
- `log_file_count` (number, default: 10): Log file count
- `custom_loggers` (array): Custom logger configurations

### Utility Functions
- `exported_utils` (array): Utility function names to export
- `utility_functions` (array): Custom utility function implementations

### Build and Deployment
- `local_run_args` (array): Arguments for local run
- `custom_make_targets` (array): Custom Makefile targets
- `additional_arguments` (array): Additional command-line arguments
  - `name` (string): Argument name
  - `help` (string): Help text
  - `default` (string): Default value

### Miscellaneous
- `license` (string, default: "Apache-2.0"): License type
- `app_purpose` (string): Application purpose description
- `default_input_path` (string): Default input path
- `default_output_path` (string): Default output path
- `additional_gitignore_patterns` (array): Additional .gitignore patterns
- `include_package_data` (boolean): Include package data
- `include_app_config` (boolean): Include application config in ConfigMap
- `include_log4j` (boolean): Include log4j configuration

## Compatibility with Scala Template

This Python template is designed to be compatible with the Scala template configuration where applicable. The following variables are shared:

- All core metadata variables
- Kubernetes configuration variables
- Docker configuration variables (with different base images)
- Spark configuration variables
- Feature flags (s3_enabled, delta_enabled, etc.)
- Data processing configuration
- Monitoring and logging configuration

## Example Usage

```yaml
# Example context for Tera template rendering
app_name: "Customer Analytics Pipeline"
organization: "com.example"
version: "1.0.0"
package_name: "customer_analytics"
main_class_name: "CustomerAnalytics"

# Python specific
python_version: "3.10"
pyspark_version: "3.5.1"

# Enable features
s3_enabled: true
delta_enabled: true
config_enabled: true
monitoring_enabled: true
use_pre_commit: true

# Configuration fields
config_fields:
  - name: "input_path"
    python_type: "str"
    default_value: "s3a://my-bucket/input"
    env_var: "INPUT_PATH"
    arg_index: 0
  - name: "output_path"
    python_type: "str"
    default_value: "s3a://my-bucket/output"
    env_var: "OUTPUT_PATH"
    arg_index: 1
  - name: "num_partitions"
    python_type: "int"
    python_default: "10"
    default_value: "10"
    env_var: "OUTPUT_PARTITIONS"

# Transformations
transformations:
  - method: "filter"
    args: ["F.col('value').isNotNull()"]
  - method: "withColumn"
    args: ["'processed_date'", "F.current_date()"]

# Docker settings
docker_registry: "myregistry.io"
namespace: "analytics"

# Resource configuration
driver_memory: "4g"
executor_memory: "4g"
executor_instances: 3
```
