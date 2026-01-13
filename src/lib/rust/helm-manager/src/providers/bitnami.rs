use crate::{Result, Values};
use serde_json::json;

/// Bitnami-specific chart configurations and helpers
pub struct BitnamiProvider;

impl BitnamiProvider {
    /// Get recommended production values for a Bitnami chart
    pub fn get_production_values(chart: &str) -> Result<Values> {
        let mut values = Values::new();

        match chart {
            "postgresql" => {
                values.set("metrics.enabled", json!(true));
                values.set("persistence.enabled", json!(true));
                values.set("resources.requests.memory", json!("256Mi"));
                values.set("resources.requests.cpu", json!("250m"));
            }
            "redis" => {
                values.set("metrics.enabled", json!(true));
                values.set("persistence.enabled", json!(true));
                values.set("replica.replicaCount", json!(2));
            }
            "kafka" => {
                values.set("metrics.kafka.enabled", json!(true));
                values.set("metrics.jmx.enabled", json!(true));
                values.set("persistence.enabled", json!(true));
                values.set("kraft.enabled", json!(true));
            }
            "elasticsearch" => {
                values.set("coordinating.replicaCount", json!(2));
                values.set("data.replicaCount", json!(3));
                values.set("master.replicaCount", json!(3));
                values.set("metrics.enabled", json!(true));
            }
            "mysql" => {
                values.set("metrics.enabled", json!(true));
                values.set("persistence.enabled", json!(true));
                values.set("architecture", json!("replication"));
            }
            "mongodb" => {
                values.set("metrics.enabled", json!(true));
                values.set("persistence.enabled", json!(true));
                values.set("architecture", json!("replicaset"));
                values.set("replicaCount", json!(3));
            }
            _ => {
                // Default production values for any chart
                values.set("resources.requests.memory", json!("128Mi"));
                values.set("resources.requests.cpu", json!("100m"));
            }
        }

        Ok(values)
    }

    /// Get minimal development values for a Bitnami chart
    pub fn get_development_values(chart: &str) -> Result<Values> {
        let mut values = Values::new();

        match chart {
            "postgresql" => {
                values.set("persistence.enabled", json!(false));
                values.set("resources.requests.memory", json!("128Mi"));
                values.set("resources.requests.cpu", json!("100m"));
            }
            "redis" => {
                values.set("persistence.enabled", json!(false));
                values.set("replica.replicaCount", json!(0));
            }
            "kafka" => {
                values.set("persistence.enabled", json!(false));
                values.set("kraft.enabled", json!(true));
                values.set("broker.replicaCount", json!(1));
            }
            "elasticsearch" => {
                values.set("coordinating.replicaCount", json!(0));
                values.set("data.replicaCount", json!(1));
                values.set("master.replicaCount", json!(1));
            }
            "mysql" => {
                values.set("persistence.enabled", json!(false));
                values.set("architecture", json!("standalone"));
            }
            "mongodb" => {
                values.set("persistence.enabled", json!(false));
                values.set("architecture", json!("standalone"));
            }
            _ => {
                // Default development values
                values.set("resources.requests.memory", json!("64Mi"));
                values.set("resources.requests.cpu", json!("50m"));
            }
        }

        Ok(values)
    }

    /// Check if a chart is a stateful service
    pub fn is_stateful(chart: &str) -> bool {
        matches!(
            chart,
            "postgresql" | "mysql" | "mongodb" | "redis" | "elasticsearch" | 
            "kafka" | "cassandra" | "mariadb" | "influxdb" | "neo4j"
        )
    }

    /// Get the default service port for a chart
    pub fn get_default_port(chart: &str) -> Option<u16> {
        match chart {
            "postgresql" => Some(5432),
            "mysql" | "mariadb" => Some(3306),
            "mongodb" => Some(27017),
            "redis" => Some(6379),
            "elasticsearch" => Some(9200),
            "kafka" => Some(9092),
            "nginx" => Some(80),
            "apache" => Some(80),
            _ => None,
        }
    }

    /// Get the selector labels for a Bitnami chart
    pub fn get_selector_labels(chart: &str, release: &str) -> String {
        format!("app.kubernetes.io/name={},app.kubernetes.io/instance={}", chart, release)
    }
}