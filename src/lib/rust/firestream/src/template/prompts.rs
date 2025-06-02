use dialoguer::{Input, Confirm, theme::ColorfulTheme};
use serde::{Serialize, Deserialize};
use crate::core::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_name: String,
    pub version: String,
    pub description: String,
    pub docker_image: String,
    pub port: u16,
    pub replicas: u32,
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
    pub enable_ingress: bool,
    pub ingress_host: Option<String>,
    pub ingress_path: Option<String>,
}

impl ProjectConfig {
    pub fn from_interactive(name: Option<String>) -> Result<Self> {
        let theme = ColorfulTheme::default();
        
        let project_name = if let Some(n) = name {
            n
        } else {
            Input::with_theme(&theme)
                .with_prompt("Project name")
                .default("my-project".to_string())
                .interact_text()?
        };
        
        let version: String = Input::with_theme(&theme)
            .with_prompt("Project version")
            .default("0.1.0".to_string())
            .interact_text()?;
        
        let description: String = Input::with_theme(&theme)
            .with_prompt("Project description")
            .default("A Helm chart for Kubernetes".to_string())
            .interact_text()?;
        
        let docker_image: String = Input::with_theme(&theme)
            .with_prompt("Docker image")
            .default("python:3.9-slim".to_string())
            .interact_text()?;
        
        let port: u16 = Input::with_theme(&theme)
            .with_prompt("Container port")
            .default(8080)
            .interact_text()?;
        
        let replicas: u32 = Input::with_theme(&theme)
            .with_prompt("Number of replicas")
            .default(1)
            .interact_text()?;
        
        let cpu_request: String = Input::with_theme(&theme)
            .with_prompt("CPU request")
            .default("100m".to_string())
            .interact_text()?;
        
        let memory_request: String = Input::with_theme(&theme)
            .with_prompt("Memory request")
            .default("128Mi".to_string())
            .interact_text()?;
        
        let cpu_limit: String = Input::with_theme(&theme)
            .with_prompt("CPU limit")
            .default("200m".to_string())
            .interact_text()?;
        
        let memory_limit: String = Input::with_theme(&theme)
            .with_prompt("Memory limit")
            .default("256Mi".to_string())
            .interact_text()?;
        
        let enable_ingress = Confirm::with_theme(&theme)
            .with_prompt("Enable ingress?")
            .default(false)
            .interact()?;
        
        let (ingress_host, ingress_path) = if enable_ingress {
            let host = Input::with_theme(&theme)
                .with_prompt("Ingress host")
                .default("example.local".to_string())
                .interact_text()?;
            
            let path = Input::with_theme(&theme)
                .with_prompt("Ingress path")
                .default("/".to_string())
                .interact_text()?;
            
            (Some(host), Some(path))
        } else {
            (None, None)
        };
        
        let confirm = Confirm::with_theme(&theme)
            .with_prompt("Proceed with these settings?")
            .default(true)
            .interact()?;
        
        if !confirm {
            return Err(crate::core::FirestreamError::GeneralError(
                "Operation cancelled by user".to_string()
            ));
        }
        
        Ok(ProjectConfig {
            project_name,
            version,
            description,
            docker_image,
            port,
            replicas,
            cpu_request,
            memory_request,
            cpu_limit,
            memory_limit,
            enable_ingress,
            ingress_host,
            ingress_path,
        })
    }
    
    pub fn from_values_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ProjectConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn default_with_name(name: Option<String>) -> Self {
        Self {
            project_name: name.unwrap_or_else(|| "my-project".to_string()),
            version: "0.1.0".to_string(),
            description: "A Helm chart for Kubernetes".to_string(),
            docker_image: "python:3.9-slim".to_string(),
            port: 8080,
            replicas: 1,
            cpu_request: "100m".to_string(),
            memory_request: "128Mi".to_string(),
            cpu_limit: "200m".to_string(),
            memory_limit: "256Mi".to_string(),
            enable_ingress: false,
            ingress_host: None,
            ingress_path: None,
        }
    }
}
