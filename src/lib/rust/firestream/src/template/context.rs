use tera::Context;
use crate::template::prompts::ProjectConfig;

pub fn build_context(config: &ProjectConfig) -> Context {
    let mut context = Context::new();
    
    // Direct mappings
    context.insert("project_name", &config.project_name);
    context.insert("replicaCount", &config.replicas);
    
    // Chart metadata
    let chart = serde_json::json!({
        "name": &config.project_name,
        "version": &config.version,
        "description": &config.description,
    });
    context.insert("chart", &chart);
    
    // Image configuration
    let (repository, tag) = config.docker_image.split_once(':')
        .unwrap_or((&config.docker_image, "latest"));
    
    let image = serde_json::json!({
        "repository": repository,
        "tag": tag,
        "pullPolicy": "IfNotPresent",
    });
    context.insert("image", &image);
    
    // Service configuration
    let service = serde_json::json!({
        "port": config.port,
    });
    context.insert("service", &service);
    
    // Resources
    let resources = serde_json::json!({
        "requests": {
            "cpu": &config.cpu_request,
            "memory": &config.memory_request,
        },
        "limits": {
            "cpu": &config.cpu_limit,
            "memory": &config.memory_limit,
        }
    });
    context.insert("resources", &resources);
    
    // Ingress
    let ingress = serde_json::json!({
        "enabled": config.enable_ingress,
        "host": config.ingress_host.as_ref().unwrap_or(&"chart-example.local".to_string()),
        "path": config.ingress_path.as_ref().unwrap_or(&"/".to_string()),
    });
    context.insert("ingress", &ingress);
    
    context
}
