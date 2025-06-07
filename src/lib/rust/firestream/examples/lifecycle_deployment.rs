//! Example of using Firestream's Helm lifecycle system programmatically
//!
//! This example demonstrates how to:
//! 1. Create and configure charts
//! 2. Deploy them with lifecycle management
//! 3. Handle results and validation
//! 4. Implement custom lifecycle logic

use firestream::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartSetValue, HelmChartNamespace,
    charts::{PrometheusOperatorChart, PostgresqlChart, NginxIngressChart},
    CommonChart, HelmAction, values_path,
};
use firestream::core::Result;
use std::path::PathBuf;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Deploy infrastructure stack
    deploy_infrastructure_stack().await?;
    
    // Deploy application services
    deploy_application_services().await?;
    
    // Deploy custom application
    deploy_custom_application().await?;
    
    info!("All deployments completed successfully!");
    Ok(())
}

/// Deploy infrastructure services (monitoring, ingress, etc.)
async fn deploy_infrastructure_stack() -> Result<()> {
    info!("Deploying infrastructure stack...");
    
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // 1. Deploy Prometheus Operator with custom configuration
    let mut prometheus = PrometheusOperatorChart::default();
    
    // Customize Prometheus configuration
    let chart_info = prometheus.get_chart_info_mut();
    chart_info.values.push(ChartSetValue {
        key: "prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage".to_string(),
        value: "50Gi".to_string(),
    });
    
    deploy_with_lifecycle("Prometheus Operator", Box::new(prometheus), &kubeconfig, &envs).await?;
    
    // 2. Deploy NGINX Ingress for K3D
    let nginx = NginxIngressChart::k3d_local();
    deploy_with_lifecycle("NGINX Ingress", Box::new(nginx), &kubeconfig, &envs).await?;
    
    // 3. Deploy External DNS (noop provider for local)
    let external_dns_info = ChartInfo {
        name: "external-dns".to_string(),
        repository: Some("bitnami".to_string()),
        chart: "external-dns".to_string(),
        version: Some("6.31.0".to_string()),
        namespace: HelmChartNamespace::KubeSystem,
        values: vec![
            ChartSetValue {
                key: "provider".to_string(),
                value: "noop".to_string(),
            },
        ],
        ..Default::default()
    };
    
    let external_dns = CommonChart::new(external_dns_info);
    deploy_with_lifecycle("External DNS", Box::new(external_dns), &kubeconfig, &envs).await?;
    
    Ok(())
}

/// Deploy application services (databases, caches, message queues)
async fn deploy_application_services() -> Result<()> {
    info!("Deploying application services...");
    
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // 1. Deploy PostgreSQL with custom values
    let mut postgres = PostgresqlChart::development();
    let chart_info = postgres.get_chart_info_mut();
    
    // Add custom values file
    chart_info.values_files.push(values_path("postgresql.yaml"));
    
    // Override specific values
    chart_info.values.push(ChartSetValue {
        key: "auth.database".to_string(),
        value: "my_application".to_string(),
    });
    
    deploy_with_lifecycle("PostgreSQL", Box::new(postgres), &kubeconfig, &envs).await?;
    
    // 2. Deploy Redis with Sentinel
    let redis_info = ChartInfo {
        name: "redis".to_string(),
        repository: Some("bitnami".to_string()),
        chart: "redis".to_string(),
        version: Some("18.6.1".to_string()),
        namespace: HelmChartNamespace::Default,
        values: vec![
            ChartSetValue {
                key: "architecture".to_string(),
                value: "replication".to_string(),
            },
            ChartSetValue {
                key: "auth.enabled".to_string(),
                value: "false".to_string(),
            },
            ChartSetValue {
                key: "sentinel.enabled".to_string(),
                value: "true".to_string(),
            },
        ],
        ..Default::default()
    };
    
    let redis = CommonChart::new(redis_info);
    deploy_with_lifecycle("Redis", Box::new(redis), &kubeconfig, &envs).await?;
    
    Ok(())
}

/// Deploy a custom application
async fn deploy_custom_application() -> Result<()> {
    info!("Deploying custom application...");
    
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // Create custom application chart
    let app_info = ChartInfo {
        name: "my-application".to_string(),
        path: Some(PathBuf::from("./charts/my-application")),
        namespace: HelmChartNamespace::Default,
        depends_on: vec!["postgresql".to_string(), "redis".to_string()],
        values: vec![
            ChartSetValue {
                key: "image.repository".to_string(),
                value: "registry.localhost:5000/my-app".to_string(),
            },
            ChartSetValue {
                key: "image.tag".to_string(),
                value: "v1.0.0".to_string(),
            },
            ChartSetValue {
                key: "replicaCount".to_string(),
                value: "3".to_string(),
            },
            ChartSetValue {
                key: "service.type".to_string(),
                value: "ClusterIP".to_string(),
            },
            ChartSetValue {
                key: "ingress.enabled".to_string(),
                value: "true".to_string(),
            },
            ChartSetValue {
                key: "ingress.className".to_string(),
                value: "nginx".to_string(),
            },
            ChartSetValue {
                key: "ingress.hosts[0].host".to_string(),
                value: "my-app.local".to_string(),
            },
            ChartSetValue {
                key: "ingress.hosts[0].paths[0].path".to_string(),
                value: "/".to_string(),
            },
            ChartSetValue {
                key: "ingress.hosts[0].paths[0].pathType".to_string(),
                value: "Prefix".to_string(),
            },
        ],
        ..Default::default()
    };
    
    let app = CommonChart::new(app_info);
    deploy_with_lifecycle("My Application", Box::new(app), &kubeconfig, &envs).await?;
    
    Ok(())
}

/// Deploy a chart with lifecycle management and handle results
async fn deploy_with_lifecycle(
    name: &str,
    chart: Box<dyn HelmChart>,
    kubeconfig: &PathBuf,
    envs: &[(String, String)],
) -> Result<()> {
    info!("Deploying {} using lifecycle management", name);
    
    match chart.run(kubeconfig, envs).await {
        Ok(Some(result)) => {
            if result.success {
                info!("✓ {} deployed successfully", name);
                
                // Log validation results
                for validation in &result.validation_results {
                    let status = if validation.passed { "✓" } else { "✗" };
                    info!("  {} {}: {}", status, validation.check_name, validation.message);
                }
                
                // Log any events if there were issues
                if !result.events.is_empty() {
                    info!("  Events:");
                    for event in &result.events {
                        info!("    {}: {} - {}", event.timestamp, event.reason, event.message);
                    }
                }
                
                Ok(())
            } else {
                error!("✗ {} deployment failed: {:?}", name, result.error);
                
                // Log failed validations
                for validation in &result.validation_results {
                    if !validation.passed {
                        error!("  ✗ {}: {}", validation.check_name, validation.message);
                    }
                }
                
                // Log events for debugging
                if !result.events.is_empty() {
                    error!("  Events:");
                    for event in &result.events {
                        error!("    {}: {} - {}", event.timestamp, event.reason, event.message);
                    }
                }
                
                Err(firestream::core::FirestreamError::GeneralError(
                    format!("{} deployment failed", name)
                ))
            }
        }
        Ok(None) => {
            info!("  {} returned no result (may have been skipped)", name);
            Ok(())
        }
        Err(e) => {
            error!("✗ Error deploying {}: {}", name, e);
            Err(e)
        }
    }
}

/// Get the kubeconfig path
fn get_kubeconfig_path() -> Result<PathBuf> {
    // Check KUBECONFIG env var first
    if let Ok(kubeconfig) = std::env::var("KUBECONFIG") {
        return Ok(PathBuf::from(kubeconfig));
    }
    
    // Fall back to default location
    let home = dirs::home_dir()
        .ok_or_else(|| firestream::core::FirestreamError::GeneralError(
            "Cannot find home directory".to_string()
        ))?;
    
    Ok(home.join(".kube").join("config"))
}

/// Example: Rollback on validation failure
#[allow(dead_code)]
async fn deploy_with_rollback() -> Result<()> {
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // Deploy PostgreSQL
    let postgres = PostgresqlChart::development();
    
    match postgres.run(&kubeconfig, &envs).await {
        Ok(Some(result)) => {
            if !result.success {
                // Check if it's a validation failure
                let validation_failed = result.validation_results.iter()
                    .any(|v| !v.passed);
                
                if validation_failed {
                    info!("Validation failed, rolling back...");
                    
                    // Create a destroy action
                    let mut rollback_chart = PostgresqlChart::development();
                    rollback_chart.get_chart_info_mut().action = HelmAction::Destroy;
                    
                    // Execute rollback
                    let _ = rollback_chart.run(&kubeconfig, &envs).await;
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// Example: Chain deployments with dependencies
#[allow(dead_code)]
async fn deploy_with_dependencies() -> Result<()> {
    let kubeconfig = get_kubeconfig_path()?;
    let envs: Vec<(String, String)> = vec![];
    
    // Define deployment order based on dependencies
    let deployments: Vec<(&str, Box<dyn HelmChart>)> = vec![
        ("PostgreSQL", Box::new(PostgresqlChart::development())),
        ("Redis", Box::new(create_redis_chart())),
        ("Application", Box::new(create_app_chart_with_deps())),
    ];
    
    // Deploy in order
    for (name, chart) in deployments {
        deploy_with_lifecycle(name, chart, &kubeconfig, &envs).await?;
        
        // Wait a bit between deployments
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    
    Ok(())
}

#[allow(dead_code)]
fn create_redis_chart() -> CommonChart {
    let redis_info = ChartInfo {
        name: "redis".to_string(),
        repository: Some("bitnami".to_string()),
        chart: "redis".to_string(),
        namespace: HelmChartNamespace::Default,
        values: vec![
            ChartSetValue {
                key: "auth.enabled".to_string(),
                value: "false".to_string(),
            },
        ],
        ..Default::default()
    };
    
    CommonChart::new(redis_info)
}

#[allow(dead_code)]
fn create_app_chart_with_deps() -> CommonChart {
    let app_info = ChartInfo {
        name: "my-app".to_string(),
        chart: "my-app".to_string(),
        namespace: HelmChartNamespace::Default,
        depends_on: vec!["postgresql".to_string(), "redis".to_string()],
        values: vec![
            ChartSetValue {
                key: "database.host".to_string(),
                value: "postgresql.default.svc.cluster.local".to_string(),
            },
            ChartSetValue {
                key: "redis.host".to_string(),
                value: "redis-master.default.svc.cluster.local".to_string(),
            },
        ],
        ..Default::default()
    };
    
    CommonChart::new(app_info)
}
