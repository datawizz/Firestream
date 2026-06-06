//! Unit tests for helm-manager.
//!
//! The crate's chart-embedding subsystem has been retired; chart discovery
//! lives in `firestream-charts`. The remaining surface here is the helm/
//! kubectl CLI wrappers and the `Deployment`/`Stack`/`Values` data types,
//! which are what these tests exercise.

use helm_manager::{Deployment, Stack, Values, models::ReleaseStatus};
use serde_json::json;

#[test]
fn test_deployment_builder() {
    let deployment = Deployment::builder("nginx")
        .name("web-server")
        .namespace("frontend")
        .atomic()
        .timeout(300)
        .build()
        .unwrap();

    assert_eq!(deployment.chart, "nginx");
    assert_eq!(deployment.name, "web-server");
    assert_eq!(deployment.namespace, "frontend");
    assert!(deployment.atomic);
    assert_eq!(deployment.timeout, 300);
}

#[test]
fn test_values_builder() {
    let mut values = Values::new();
    values.set("image.tag", json!("latest"));
    values.set("service.port", json!(8080));
    values.set("resources.requests.memory", json!("256Mi"));

    assert_eq!(values.get("image.tag"), Some(json!("latest")));
    assert_eq!(values.get("service.port"), Some(json!(8080)));
}

#[test]
fn test_stack_builder() {
    let stack = Stack::builder()
        .name("app-stack")
        .namespace("production")
        .add("postgresql", |d| d.name("database"))
        .add("redis", |d| d.name("cache"))
        .add("elasticsearch", |d| d.name("search"))
        .build()
        .unwrap();

    assert_eq!(stack.name, "app-stack");
    assert_eq!(stack.namespace, "production");
    assert_eq!(stack.deployments.len(), 3);
}

#[test]
fn test_release_status_parsing() {
    use std::str::FromStr;

    assert_eq!(
        ReleaseStatus::from_str("deployed").unwrap(),
        ReleaseStatus::Deployed
    );
    assert_eq!(
        ReleaseStatus::from_str("failed").unwrap(),
        ReleaseStatus::Failed
    );
    assert_eq!(
        ReleaseStatus::from_str("pending-install").unwrap(),
        ReleaseStatus::PendingInstall
    );
    assert_eq!(
        ReleaseStatus::from_str("unknown-status").unwrap(),
        ReleaseStatus::Unknown
    );
}

#[test]
fn test_values_merge() {
    let mut base = Values::new();
    base.set("app.name", json!("myapp"));
    base.set("app.port", json!(8080));
    base.set("app.replicas", json!(1));

    let mut override_values = Values::new();
    override_values.set("app.port", json!(9090));
    override_values.set("app.debug", json!(true));

    base.merge(override_values);

    assert_eq!(base.get("app.name"), Some(json!("myapp")));
    assert_eq!(base.get("app.port"), Some(json!(9090)));
    assert_eq!(base.get("app.replicas"), Some(json!(1)));
    assert_eq!(base.get("app.debug"), Some(json!(true)));
}

#[test]
fn test_dependency_validation() {
    // Valid dependencies
    let valid_stack = Stack::builder()
        .add("postgresql", |d| d.name("db"))
        .add("app", |d| d.name("myapp").depends_on("db"))
        .build()
        .unwrap();

    assert!(valid_stack.validate_dependencies().is_ok());

    // Invalid dependencies
    let invalid_stack = Stack::builder()
        .add("app", |d| d.name("myapp").depends_on("missing"))
        .build()
        .unwrap();

    assert!(invalid_stack.validate_dependencies().is_err());
}
