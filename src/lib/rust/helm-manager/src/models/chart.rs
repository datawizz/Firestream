use serde::{Deserialize, Serialize};

/// Represents a Helm chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Chart name
    pub name: String,
    
    /// Chart version
    pub version: String,
    
    /// Chart description
    pub description: Option<String>,
    
    /// Chart metadata
    pub metadata: ChartMetadata,
    
    /// Chart dependencies
    pub dependencies: Vec<ChartDependency>,
}

/// Chart metadata from Chart.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartMetadata {
    /// API version
    pub api_version: String,
    
    /// Chart name
    pub name: String,
    
    /// Chart version
    pub version: String,
    
    /// App version
    pub app_version: Option<String>,
    
    /// Chart description
    pub description: Option<String>,
    
    /// Chart type (application or library)
    #[serde(rename = "type")]
    pub chart_type: Option<String>,
    
    /// Keywords
    pub keywords: Option<Vec<String>>,
    
    /// Home URL
    pub home: Option<String>,
    
    /// Icon URL
    pub icon: Option<String>,
    
    /// Sources
    pub sources: Option<Vec<String>>,
    
    /// Maintainers
    pub maintainers: Option<Vec<Maintainer>>,
    
    /// Annotations
    pub annotations: Option<serde_json::Value>,
}

/// Chart maintainer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    /// Name
    pub name: String,
    
    /// Email
    pub email: Option<String>,
    
    /// URL
    pub url: Option<String>,
}

/// Chart dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDependency {
    /// Dependency name
    pub name: String,
    
    /// Version constraint
    pub version: String,
    
    /// Repository URL
    pub repository: Option<String>,
    
    /// Condition for enabling the dependency
    pub condition: Option<String>,
    
    /// Tags
    pub tags: Option<Vec<String>>,
    
    /// Import values
    #[serde(rename = "import-values")]
    pub import_values: Option<Vec<String>>,
    
    /// Alias
    pub alias: Option<String>,
}

impl Chart {
    /// Check if this is a Bitnami chart
    pub fn is_bitnami(&self) -> bool {
        self.metadata.sources.as_ref()
            .map(|sources| sources.iter().any(|s| s.contains("bitnami")))
            .unwrap_or(false)
    }
    
    /// Get the common dependency if it exists
    pub fn get_common_dependency(&self) -> Option<&ChartDependency> {
        self.dependencies.iter().find(|d| d.name == "common")
    }
}