use crate::{Deployment, Error, Result, Values};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;

/// Resolve values for a deployment from various sources.
///
/// Precedence (lowest to highest, later sources override earlier ones):
/// 1. Values files (in the order they were added).
/// 2. Environment variables / `.env` file.
/// 3. Inline values set on the [`Deployment`].
///
/// Default chart values used to be sourced from an in-crate embedded chart
/// archive; that responsibility has moved to the `firestream-charts` crate.
/// Callers that want chart defaults applied should add the chart's
/// `values.yaml` to `values_files` explicitly.
pub async fn resolve_values(deployment: &Deployment) -> Result<JsonValue> {
    let mut values = Values::new();

    // 1. Apply values from files (in order)
    for values_file in &deployment.values_files {
        let file_values = load_values_file(values_file).await?;
        values.merge(file_values);
    }

    // 2. Apply environment variables
    if let Some(ref env_file) = deployment.env_file {
        let env_values = load_env_values(env_file, &deployment.env_prefix).await?;
        values.merge(env_values);
    }

    // 3. Apply inline values (highest priority)
    if let Some(ref inline_values) = deployment.values {
        values.merge(inline_values.clone());
    }

    Ok(values.to_json())
}

/// Load values from a YAML file
async fn load_values_file(path: impl AsRef<Path>) -> Result<Values> {
    let content = tokio::fs::read_to_string(path.as_ref()).await
        .map_err(|e| Error::InvalidValues(format!("Failed to read values file: {}", e)))?;
    
    Values::from_yaml(&content)
}

/// Load values from environment variables
async fn load_env_values(env_file: impl AsRef<Path>, prefix: &Option<String>) -> Result<Values> {
    let mut env_map = HashMap::new();

    // Load from .env file
    let env_path = env_file.as_ref();
    if env_path.exists() {
        match dotenvy::from_path_iter(env_path) {
            Ok(iter) => {
                for item in iter {
                    if let Ok((key, value)) = item {
                        env_map.insert(key, value);
                    }
                }
            }
            Err(e) => {
                return Err(Error::EnvFile(format!(
                    "Failed to load environment file {}: {}",
                    env_path.display(),
                    e
                )));
            }
        }
    }

    // Also include current environment variables
    for (key, value) in std::env::vars() {
        env_map.insert(key, value);
    }

    // Convert environment variables to values
    let values = convert_env_to_values(env_map, prefix)?;
    
    Ok(values)
}

/// Convert environment variables to Helm values
fn convert_env_to_values(env_map: HashMap<String, String>, prefix: &Option<String>) -> Result<Values> {
    let mut values = Values::new();
    
    // Common env-var → Helm-values path mappings used by many charts.
    let mappings = get_chart_env_mappings();
    
    for (env_key, env_value) in env_map {
        // Check if this env var should be included
        if let Some(prefix) = prefix {
            if !env_key.starts_with(prefix) {
                continue;
            }
        }
        
        // Try to find a mapping for this environment variable
        if let Some(helm_path) = mappings.get(&env_key) {
            let json_value = parse_env_value(&env_value);
            values.set(helm_path, json_value);
        } else if prefix.is_some() {
            // For prefixed variables, convert to helm path
            let helm_path = env_key_to_helm_path(&env_key, prefix.as_ref().unwrap());
            let json_value = parse_env_value(&env_value);
            values.set(&helm_path, json_value);
        }
    }
    
    Ok(values)
}

/// Common env-var → Helm-values path mappings shared across standard charts.
fn get_chart_env_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();
    
    // PostgreSQL mappings
    mappings.insert("POSTGRES_USER".to_string(), "postgresql.auth.username".to_string());
    mappings.insert("POSTGRES_PASSWORD".to_string(), "postgresql.auth.password".to_string());
    mappings.insert("POSTGRES_DB".to_string(), "postgresql.auth.database".to_string());
    mappings.insert("POSTGRES_DEFAULT_DB".to_string(), "postgresql.auth.database".to_string());
    mappings.insert("PGUSER".to_string(), "postgresql.auth.username".to_string());
    mappings.insert("PGPASSWORD".to_string(), "postgresql.auth.password".to_string());
    mappings.insert("PGDATABASE".to_string(), "postgresql.auth.database".to_string());
    
    // Kafka mappings
    mappings.insert("KAFKA_BOOTSTRAP_SERVERS".to_string(), "kafka.bootstrapServers".to_string());
    mappings.insert("KAFKA_BOOTSTRAP_SERVER".to_string(), "kafka.bootstrapServers".to_string());
    
    // Redis mappings
    mappings.insert("REDIS_PASSWORD".to_string(), "redis.auth.password".to_string());
    mappings.insert("REDIS_MASTER_PASSWORD".to_string(), "redis.auth.password".to_string());
    
    // MySQL mappings
    mappings.insert("MYSQL_ROOT_PASSWORD".to_string(), "mysql.auth.rootPassword".to_string());
    mappings.insert("MYSQL_USER".to_string(), "mysql.auth.username".to_string());
    mappings.insert("MYSQL_PASSWORD".to_string(), "mysql.auth.password".to_string());
    mappings.insert("MYSQL_DATABASE".to_string(), "mysql.auth.database".to_string());
    
    // MongoDB mappings
    mappings.insert("MONGODB_ROOT_PASSWORD".to_string(), "mongodb.auth.rootPassword".to_string());
    mappings.insert("MONGODB_USERNAME".to_string(), "mongodb.auth.username".to_string());
    mappings.insert("MONGODB_PASSWORD".to_string(), "mongodb.auth.password".to_string());
    mappings.insert("MONGODB_DATABASE".to_string(), "mongodb.auth.database".to_string());
    
    // Elasticsearch mappings
    mappings.insert("ELASTICSEARCH_USERNAME".to_string(), "elasticsearch.auth.username".to_string());
    mappings.insert("ELASTICSEARCH_PASSWORD".to_string(), "elasticsearch.auth.password".to_string());
    
    // MinIO mappings
    mappings.insert("MINIO_ACCESS_KEY".to_string(), "minio.auth.rootUser".to_string());
    mappings.insert("MINIO_SECRET_KEY".to_string(), "minio.auth.rootPassword".to_string());
    mappings.insert("S3_LOCAL_ACCESS_KEY_ID".to_string(), "minio.auth.rootUser".to_string());
    mappings.insert("S3_LOCAL_SECRET_ACCESS_KEY".to_string(), "minio.auth.rootPassword".to_string());
    
    // Common mappings
    mappings.insert("REPLICA_COUNT".to_string(), "replicaCount".to_string());
    mappings.insert("SERVICE_TYPE".to_string(), "service.type".to_string());
    mappings.insert("SERVICE_PORT".to_string(), "service.port".to_string());
    
    mappings
}

/// Convert environment variable key to Helm values path
fn env_key_to_helm_path(env_key: &str, prefix: &str) -> String {
    // Remove prefix
    let key = env_key.strip_prefix(prefix).unwrap_or(env_key);
    
    // Convert to lowercase and replace underscores with dots
    key.to_lowercase().replace('_', ".")
}

/// Parse environment variable value to appropriate JSON type
fn parse_env_value(value: &str) -> JsonValue {
    // Try to parse as boolean
    if let Ok(bool_val) = value.parse::<bool>() {
        return JsonValue::Bool(bool_val);
    }
    
    // Try to parse as number
    if let Ok(int_val) = value.parse::<i64>() {
        return JsonValue::Number(int_val.into());
    }
    
    if let Ok(float_val) = value.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return JsonValue::Number(num);
        }
    }
    
    // Try to parse as JSON array or object
    if (value.starts_with('[') && value.ends_with(']')) || 
       (value.starts_with('{') && value.ends_with('}')) {
        if let Ok(json_val) = serde_json::from_str(value) {
            return json_val;
        }
    }
    
    // Default to string
    JsonValue::String(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_value() {
        assert_eq!(parse_env_value("true"), JsonValue::Bool(true));
        assert_eq!(parse_env_value("false"), JsonValue::Bool(false));
        assert_eq!(parse_env_value("42"), JsonValue::Number(42.into()));
        assert_eq!(parse_env_value("3.14"), JsonValue::Number(serde_json::Number::from_f64(3.14).unwrap()));
        assert_eq!(parse_env_value("hello"), JsonValue::String("hello".to_string()));
        assert_eq!(
            parse_env_value("[1,2,3]"),
            serde_json::json!([1, 2, 3])
        );
    }

    #[test]
    fn test_env_key_to_helm_path() {
        assert_eq!(env_key_to_helm_path("POSTGRES_USER", "POSTGRES_"), "user");
        assert_eq!(env_key_to_helm_path("KAFKA_BOOTSTRAP_SERVERS", "KAFKA_"), "bootstrap.servers");
    }
}