use std::path::{Path, PathBuf};
use tokio::fs;
use crate::template::{
    engine::TemplateEngine, 
    prompts::ProjectConfig, 
    context::build_context,
    embedded::{TEMPLATES, STATIC_FILES}
};
use crate::core::Result;

pub struct TemplateProcessor {
    engine: TemplateEngine,
    output_dir: PathBuf,
}

impl TemplateProcessor {
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        let engine = TemplateEngine::new()?;
        Ok(Self { engine, output_dir })
    }
    
    pub async fn process(&self, config: &ProjectConfig, project_type: &str) -> Result<()> {
        let context = build_context(config);
        let project_dir = self.output_dir.join(&config.project_name);
        
        // Check if project directory already exists
        if project_dir.exists() {
            return Err(crate::core::FirestreamError::GeneralError(
                format!("Directory '{}' already exists", project_dir.display())
            ));
        }
        
        // Create directory structure
        self.create_directory_structure(&project_dir, &config.project_name).await?;
        
        // Process Tera templates
        for (template_path, _) in TEMPLATES.iter() {
            // Skip templates not relevant to the project type
            if !self.should_include_file(template_path, project_type) {
                continue;
            }
            
            let output_path = self.resolve_output_path(&project_dir, template_path, &config.project_name);
            let rendered = self.engine.render(template_path, &context)?;
            
            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            
            fs::write(&output_path, rendered).await?;
        }
        
        // Copy static files
        for (file_path, content) in STATIC_FILES.iter() {
            // Skip files not relevant to the project type
            if !self.should_include_file(file_path, project_type) {
                continue;
            }
            
            let output_path = self.resolve_output_path(&project_dir, file_path, &config.project_name);
            
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            
            fs::write(&output_path, content).await?;
        }
        
        // Post-process Helm templates to replace "new_project_template" with actual project name
        self.post_process_helm_templates(&project_dir, &config.project_name).await?;
        
        // Set executable permissions for scripts
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let scripts = ["bootstrap.sh", "entrypoint.sh"];
            for script in &scripts {
                let script_path = project_dir.join(script);
                if script_path.exists() {
                    let mut perms = fs::metadata(&script_path).await?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&script_path, perms).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn create_directory_structure(&self, project_dir: &Path, project_name: &str) -> Result<()> {
        let dirs = vec![
            project_dir.to_path_buf(),
            project_dir.join("helm/template/charts"),
            project_dir.join("helm/template/templates/tests"),
            project_dir.join(format!("docker/{}", project_name)),
            project_dir.join("src"),
            project_dir.join("tests"),
            project_dir.join(".devcontainer"),
        ];
        
        for dir in dirs {
            fs::create_dir_all(dir).await?;
        }
        
        Ok(())
    }
    
    fn should_include_file(&self, path: &str, project_type: &str) -> bool {
        match project_type {
            "python-fastapi" | "helm" => true, // Include everything for helm/python-fastapi projects
            "kubernetes" => !path.starts_with("docker/"), // Exclude docker files
            "docker" => !path.starts_with("helm/"), // Exclude helm files
            _ => true,
        }
    }
    
    fn resolve_output_path(&self, project_dir: &Path, template_path: &str, project_name: &str) -> PathBuf {
        // Remove .tera extension if present
        let path = template_path.trim_end_matches(".tera");
        
        // Handle special cases and path mappings
        match path {
            // Helm files go into helm/template/
            "helm/Chart.yaml" => project_dir.join("helm/template/Chart.yaml"),
            "helm/values.yaml" => project_dir.join("helm/template/values.yaml"),
            "helm/.helmignore" => project_dir.join("helm/template/.helmignore"),
            
            // Helm templates maintain their structure
            path if path.starts_with("helm/templates/") => {
                let relative = path.strip_prefix("helm/").unwrap();
                project_dir.join("helm/template").join(relative)
            }
            
            // Docker files go into docker/{project_name}/
            "docker/Dockerfile" => project_dir.join(format!("docker/{}/Dockerfile", project_name)),
            
            // Everything else goes to project root
            _ => project_dir.join(path),
        }
    }
    
    /// Post-process Helm templates to replace placeholder project name
    async fn post_process_helm_templates(&self, project_dir: &Path, project_name: &str) -> Result<()> {
        let helm_templates_dir = project_dir.join("helm/template/templates");
        
        // List of files to process
        let files_to_process = vec![
            "deployment.yaml",
            "service.yaml",
            "ingress.yaml",
            "hpa.yaml",
            "serviceaccount.yaml",
            "_helpers.tpl",
            "NOTES.txt",
            "tests/test-connection.yaml",
        ];
        
        for file in files_to_process {
            let file_path = helm_templates_dir.join(file);
            if file_path.exists() {
                // Read the file
                let content = fs::read_to_string(&file_path).await?;
                
                // Replace all occurrences of "new_project_template" with the actual project name
                let updated_content = content.replace("new_project_template", project_name);
                
                // Write back if changed
                if content != updated_content {
                    fs::write(&file_path, updated_content).await?;
                }
            }
        }
        
        Ok(())
    }
}
