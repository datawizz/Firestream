//! PostgreSQL chart implementation

use crate::deploy::helm_lifecycle::{
    ChartInfo, ChartSetValue, HelmChartNamespace,
    CommonChart, values_path, HelmChart,
};

/// PostgreSQL chart configuration
pub struct PostgresqlChart;

impl PostgresqlChart {
    /// Create PostgreSQL chart with default configuration
    pub fn default() -> CommonChart {
        let chart_info = ChartInfo {
            name: "postgresql".to_string(),
            repository: Some("bitnami".to_string()),
            chart: "postgresql".to_string(),
            version: Some("13.2.24".to_string()),
            namespace: HelmChartNamespace::Default,
            values_files: vec![values_path("postgresql.yaml")],
            values: vec![
                // Authentication
                ChartSetValue {
                    key: "auth.enablePostgresUser".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "auth.postgresPassword".to_string(),
                    value: "postgres".to_string(),
                },
                ChartSetValue {
                    key: "auth.database".to_string(),
                    value: "firestream".to_string(),
                },
                // Resources
                ChartSetValue {
                    key: "primary.resources.limits.cpu".to_string(),
                    value: "500m".to_string(),
                },
                ChartSetValue {
                    key: "primary.resources.requests.cpu".to_string(),
                    value: "250m".to_string(),
                },
                ChartSetValue {
                    key: "primary.resources.limits.memory".to_string(),
                    value: "512Mi".to_string(),
                },
                ChartSetValue {
                    key: "primary.resources.requests.memory".to_string(),
                    value: "256Mi".to_string(),
                },
                // Persistence
                ChartSetValue {
                    key: "primary.persistence.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "primary.persistence.size".to_string(),
                    value: "10Gi".to_string(),
                },
                // Metrics
                ChartSetValue {
                    key: "metrics.enabled".to_string(),
                    value: "true".to_string(),
                },
                ChartSetValue {
                    key: "metrics.serviceMonitor.enabled".to_string(),
                    value: "true".to_string(),
                },
            ],
            ..Default::default()
        };
        
        CommonChart::new(chart_info)
    }
    
    /// Create PostgreSQL for development (smaller resources)
    pub fn development() -> CommonChart {
        let mut chart = Self::default();
        let info = chart.get_chart_info_mut();
        
        // Override resources for development
        info.values.retain(|v| !v.key.contains("resources"));
        info.values.extend(vec![
            ChartSetValue {
                key: "primary.resources.limits.cpu".to_string(),
                value: "200m".to_string(),
            },
            ChartSetValue {
                key: "primary.resources.requests.cpu".to_string(),
                value: "100m".to_string(),
            },
            ChartSetValue {
                key: "primary.resources.limits.memory".to_string(),
                value: "256Mi".to_string(),
            },
            ChartSetValue {
                key: "primary.resources.requests.memory".to_string(),
                value: "128Mi".to_string(),
            },
        ]);
        
        // Smaller disk for development
        info.values.retain(|v| v.key != "primary.persistence.size");
        info.values.push(ChartSetValue {
            key: "primary.persistence.size".to_string(),
            value: "1Gi".to_string(),
        });
        
        chart
    }
    
    /// Create PostgreSQL for production (HA configuration)
    pub fn production() -> CommonChart {
        let mut chart = Self::default();
        let info = chart.get_chart_info_mut();
        
        // Enable replication
        info.values.push(ChartSetValue {
            key: "architecture".to_string(),
            value: "replication".to_string(),
        });
        
        // Configure read replicas
        info.values.extend(vec![
            ChartSetValue {
                key: "readReplicas.replicaCount".to_string(),
                value: "2".to_string(),
            },
            ChartSetValue {
                key: "readReplicas.resources.limits.cpu".to_string(),
                value: "500m".to_string(),
            },
            ChartSetValue {
                key: "readReplicas.resources.requests.cpu".to_string(),
                value: "250m".to_string(),
            },
            ChartSetValue {
                key: "readReplicas.resources.limits.memory".to_string(),
                value: "512Mi".to_string(),
            },
            ChartSetValue {
                key: "readReplicas.resources.requests.memory".to_string(),
                value: "256Mi".to_string(),
            },
        ]);
        
        // Larger storage for production
        info.values.retain(|v| v.key != "primary.persistence.size");
        info.values.push(ChartSetValue {
            key: "primary.persistence.size".to_string(),
            value: "100Gi".to_string(),
        });
        
        // Enable backups
        info.values.push(ChartSetValue {
            key: "backup.enabled".to_string(),
            value: "true".to_string(),
        });
        
        chart
    }
}
