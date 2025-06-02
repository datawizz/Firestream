use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Embedded template files that need Tera processing
pub static TEMPLATES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut templates = HashMap::new();
    
    // Only Chart.yaml and values.yaml need Tera templating
    templates.insert("helm/Chart.yaml", include_str!("../templates/_python_fastapi/helm/template/Chart.yaml.tera"));
    templates.insert("helm/values.yaml", include_str!("../templates/_python_fastapi/helm/template/values.yaml.tera"));
    
    templates
});

/// Static files that are copied as-is
pub static STATIC_FILES: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {
    let mut files: HashMap<&'static str, &'static [u8]> = HashMap::new();
    
    // Helm templates (these use Helm's own templating, not Tera)
    files.insert("helm/templates/deployment.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/deployment.yaml") as &[u8]);
    files.insert("helm/templates/service.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/service.yaml") as &[u8]);
    files.insert("helm/templates/ingress.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/ingress.yaml") as &[u8]);
    files.insert("helm/templates/hpa.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/hpa.yaml") as &[u8]);
    files.insert("helm/templates/serviceaccount.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/serviceaccount.yaml") as &[u8]);
    files.insert("helm/templates/_helpers.tpl", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/_helpers.tpl") as &[u8]);
    files.insert("helm/templates/NOTES.txt", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/NOTES.txt") as &[u8]);
    files.insert("helm/templates/tests/test-connection.yaml", 
        include_bytes!("../templates/_python_fastapi/helm/template/templates/tests/test-connection.yaml") as &[u8]);
    files.insert("helm/.helmignore", 
        include_bytes!("../templates/_python_fastapi/helm/template/.helmignore") as &[u8]);
    
    // Docker files
    files.insert("docker/Dockerfile", 
        include_bytes!("../templates/_python_fastapi/docker/my-project/Dockerfile") as &[u8]);
    
    // Project files
    files.insert("README.md", 
        include_bytes!("../templates/_python_fastapi/README.md") as &[u8]);
    files.insert("Makefile", 
        include_bytes!("../templates/_python_fastapi/makefile") as &[u8]);
    files.insert("bootstrap.sh", 
        include_bytes!("../templates/_python_fastapi/bootstrap.sh") as &[u8]);
    files.insert("entrypoint.sh", 
        include_bytes!("../templates/_python_fastapi/entrypoint.sh") as &[u8]);
    files.insert(".dockerignore", 
        include_bytes!("../templates/_python_fastapi/.dockerignore") as &[u8]);
    
    // Source files
    files.insert("src/main.py", 
        include_bytes!("../templates/_python_fastapi/src/main.py") as &[u8]);
    
    // Python requirements
    files.insert("requirements.txt", 
        include_bytes!("../templates/_python_fastapi/requirements.txt") as &[u8]);
    
    // Test files
    files.insert("tests/test.py", 
        include_bytes!("../templates/_python_fastapi/tests/test.py") as &[u8]);
    
    // DevContainer
    files.insert(".devcontainer/devcontainer.json", 
        include_bytes!("../templates/_python_fastapi/.devcontainer/devcontainer.json") as &[u8]);
    
    files
});
