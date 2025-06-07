# Spark Scala Boilerplate Template Variables

This document describes all available variables that can be used with the Spark Scala boilerplate templates.

## Core Variables

### Application Metadata
- `app_name` (string, required): Name of the Spark application
- `organization` (string, required): Organization name for the project
- `version` (string, required): Application version
- `description` (string): Application description
- `package_name` (string, required): Scala package name
- `main_class_name` (string, default: "SparkApp"): Main class name

### Build Configuration
- `scala_version` (string, default: "2.13.15"): Scala version
- `scala_major_version` (string, default: "2.13"): Scala major version
- `spark_version` (string, default: "3.5.1"): Apache Spark version
- `sbt_version` (string, default: "1.9.7"): SBT version
- `sbt_assembly_version` (string, default: "2.1.5"): sbt-assembly plugin version
- `java_version` (string, default: "11"): Java version

### Features Flags
- `streaming_enabled` (boolean): Include Spark Streaming dependencies
- `mllib_enabled` (boolean): Include Spark MLlib dependencies
- `s3_enabled` (boolean): Include S3/AWS dependencies
- `delta_enabled` (boolean): Include Delta Lake support
- `config_enabled` (boolean): Include configuration management
- `checkpoint_enabled` (boolean): Enable checkpoint directory support
- `cache_input` (boolean): Cache input DataFrame
- `include_utils` (boolean): Include utility object
- `include_integration_test` (boolean): Include integration tests
- `monitoring_enabled` (boolean): Enable Prometheus monitoring
- `file_logging` (boolean): Enable file-based logging
- `logging_config` (boolean): Include logging configuration

### Data Processing
- `input_format` (string): Input data format (json, parquet, csv, delta)
- `output_format` (string): Output data format (json, parquet, csv, delta)
- `output_mode` (string, default: "overwrite"): Output write mode
- `compression` (string, default: "snappy"): Compression codec
- `transformations` (array): List of transformations to apply
  - `method` (string): Transformation method name
  - `args` (array): Method arguments
- `aggregations` (array): List of aggregations
  - `function` (string): Aggregation function
  - `column` (string): Column to aggregate
  - `alias` (string): Output column alias
- `group_by_columns` (array): Columns for groupBy operation

### Configuration Fields
- `config_fields` (array): Application configuration fields
  - `name` (string): Field name
  - `type` (string): Scala type
  - `config_type` (string): Config type (String, Int, etc.)
  - `config_path` (string): Path in config file
  - `config_key` (string): Key in config file
  - `config_section` (string): Config section
  - `default_value` (string): Default value
  - `env_var` (string): Environment variable name
  - `arg_index` (number): Command-line argument index
  - `description` (string): Field description

### Spark Configuration
- `spark_configs` (array): Spark configuration settings
  - `key` (string): Configuration key
  - `value` (string): Configuration value
  - `env_var` (string): Environment variable override
- `spark_conf` (map): Spark configuration map for K8s
- `hadoop_conf` (map): Hadoop configuration map

### Docker Configuration
- `docker_registry` (string, required): Docker registry URL
- `spark_base_image` (string): Base Spark Docker image
- `image_pull_policy` (string, default: "Always"): Image pull policy
- `image_pull_secrets` (array): Image pull secrets
- `apt_packages` (array): Additional apt packages to install
- `additional_jars` (array): Additional JAR files to include
- `config_files` (array): Configuration files to copy
  - `source` (string): Source path
  - `destination` (string): Destination path
- `environment_vars` (array): Environment variables
  - `name` (string): Variable name
  - `value` (string): Variable value

### Kubernetes Configuration
- `namespace` (string, default: "spark-apps"): Kubernetes namespace
- `service_account` (string, default: "spark"): Service account name
- `labels` (map): Additional labels
- `arguments` (array): Application arguments
- `restart_policy_type` (string, default: "OnFailure"): Restart policy
- `on_failure_retries` (number, default: 3): Failure retry count
- `on_failure_retry_interval` (number, default: 10): Retry interval
- `on_submission_failure_retries` (number, default: 5): Submission failure retries
- `on_submission_failure_retry_interval` (number, default: 20): Submission retry interval

### Driver Configuration
- `driver_cores` (number, default: 1): Driver CPU cores
- `driver_core_limit` (string, default: "1200m"): Driver CPU limit
- `driver_memory` (string, default: "2g"): Driver memory
- `driver_labels` (map): Driver pod labels
- `driver_env_vars` (array): Driver environment variables
- `driver_volume_mounts` (array): Driver volume mounts
- `driver_node_selector` (map): Driver node selector
- `driver_tolerations` (array): Driver tolerations

### Executor Configuration
- `executor_cores` (number, default: 1): Executor CPU cores
- `executor_instances` (number, default: 2): Number of executors
- `executor_memory` (string, default: "2g"): Executor memory
- `executor_labels` (map): Executor pod labels
- `executor_env_vars` (array): Executor environment variables
- `executor_volume_mounts` (array): Executor volume mounts
- `executor_node_selector` (map): Executor node selector
- `executor_tolerations` (array): Executor tolerations

### Dynamic Allocation
- `dynamic_allocation` (object): Dynamic allocation settings
  - `initial_executors` (number): Initial executor count
  - `min_executors` (number): Minimum executors
  - `max_executors` (number): Maximum executors

### Volumes
- `volumes` (array): Kubernetes volumes
  - `name` (string): Volume name
  - `configMap` (object): ConfigMap volume source
  - `secret` (object): Secret volume source
  - `persistentVolumeClaim` (object): PVC volume source
  - `emptyDir` (object): EmptyDir volume source

### Monitoring
- `spark_ui_port` (string, default: "4040"): Spark UI port
- `spark_event_log_enabled` (boolean): Enable event logging
- `spark_event_log_dir` (string): Event log directory
- `prometheus_port` (number, default: 8090): Prometheus metrics port
- `jmx_exporter_version` (string, default: "0.11.0"): JMX exporter version

### Logging
- `log_level` (string, default: "INFO"): Log level
- `log_file_path` (string): Log file path
- `log_file_size` (string, default: "100MB"): Log file size
- `log_file_count` (number, default: 10): Log file count
- `custom_loggers` (array): Custom logger configurations
  - `name` (string): Logger name
  - `level` (string): Log level

### Dependencies
- `hadoop_version` (string): Hadoop version
- `delta_version` (string, default: "2.4.0"): Delta Lake version
- `additional_dependencies` (array): Additional dependencies
  - `group` (string): Group ID
  - `artifact` (string): Artifact ID
  - `version` (string): Version
  - `scala_version` (boolean): Use %% for Scala version
  - `provided` (boolean): Mark as provided
- `dependency_overrides` (array): Dependency overrides
- `additional_plugins` (array): Additional SBT plugins

### Testing
- `test_sample_data` (array): Sample data for tests
- `test_columns` (array): Test DataFrame columns
- `additional_tests` (array): Additional test cases
  - `name` (string): Test name
  - `body` (string): Test body

### RBAC
- `additional_rbac_rules` (array): Additional RBAC rules
  - `apiGroups` (array): API groups
  - `resources` (array): Resources
  - `verbs` (array): Verbs

### ConfigMap
- `include_log4j` (boolean): Include log4j.properties
- `include_app_conf` (boolean): Include application.conf
- `additional_config_files` (array): Additional config files
  - `name` (string): File name
  - `content` (string): File content

### Miscellaneous
- `license` (string, default: "Apache 2.0"): License type
- `app_purpose` (string): Application purpose description
- `additional_imports` (array): Additional Scala imports
- `utility_functions` (array): Utility function implementations
- `additional_gitignore_patterns` (array): Additional .gitignore patterns
- `custom_make_targets` (array): Custom Makefile targets
  - `name` (string): Target name
  - `description` (string): Target description
  - `dependencies` (array): Target dependencies
  - `commands` (array): Commands to execute
- `local_run_args` (array): Arguments for local run

## Example Usage

```yaml
# Example context for Tera template rendering
app_name: "Customer Analytics Pipeline"
organization: "com.example"
version: "1.0.0"
package_name: "com.example.analytics"
main_class_name: "CustomerAnalytics"

# Enable features
s3_enabled: true
delta_enabled: true
config_enabled: true
monitoring_enabled: true

# Configuration fields
config_fields:
  - name: "inputPath"
    type: "String"
    config_type: "String"
    config_path: "input.path"
    default_value: "s3a://my-bucket/input"
    env_var: "INPUT_PATH"
    arg_index: 0
  - name: "outputPath"
    type: "String"
    config_type: "String"
    config_path: "output.path"
    default_value: "s3a://my-bucket/output"
    env_var: "OUTPUT_PATH"
    arg_index: 1

# Spark configuration
spark_configs:
  - key: "spark.sql.adaptive.enabled"
    value: "true"
  - key: "spark.sql.adaptive.coalescePartitions.enabled"
    value: "true"

# Docker settings
docker_registry: "myregistry.io"
namespace: "analytics"

# Resource configuration
driver_memory: "4g"
executor_memory: "4g"
executor_instances: 3
```
