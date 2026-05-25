//! Configuration types for Odoo addons project generation

use crate::config::TemplateConfig;
use crate::error::{Result, TemplatizerError};
use serde::{Deserialize, Serialize};
use tera::Context;

/// Map Odoo major version to the Python version used by Firestream containers.
/// These match the actual flake.nix pythonForVersion mapping.
pub fn python_for_odoo_version(version: u8) -> &'static str {
    match version {
        15 | 16 => "3.10",
        17 => "3.11",
        18 => "3.12",
        _ => "3.12",
    }
}

/// Configuration for Odoo addons project generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdooConfig {
    /// Project display name
    pub project_name: String,

    /// Project description
    #[serde(default = "default_description")]
    pub description: String,

    /// Organization name
    #[serde(default)]
    pub organization: String,

    /// Addon author
    #[serde(default)]
    pub author: String,

    /// Author email
    #[serde(default)]
    pub author_email: String,

    /// License identifier
    #[serde(default = "default_license")]
    pub license: String,

    /// Odoo major version (15, 16, 17, or 18)
    #[serde(default = "default_odoo_version")]
    pub odoo_version: u8,

    /// Host HTTP port
    #[serde(default = "default_http_port")]
    pub odoo_http_port: u16,

    /// Host longpolling port
    #[serde(default = "default_longpolling_port")]
    pub odoo_longpolling_port: u16,

    /// Odoo admin login email
    #[serde(default = "default_admin_email")]
    pub odoo_admin_email: String,

    /// Odoo admin password
    #[serde(default = "default_admin_password")]
    pub odoo_admin_password: String,

    /// PostgreSQL version
    #[serde(default = "default_postgresql_version")]
    pub postgresql_version: String,

    /// Host PostgreSQL port
    #[serde(default = "default_postgresql_port")]
    pub postgresql_port: u16,

    /// Database name
    #[serde(default = "default_database_name")]
    pub database_name: String,

    /// Database user
    #[serde(default = "default_database_user")]
    pub database_user: String,

    /// Database password
    #[serde(default = "default_database_password")]
    pub database_password: String,

    /// Nix flake input URL for Firestream
    #[serde(default = "default_firestream_ref")]
    pub firestream_ref: String,

    /// Whether to load Odoo demo data on init
    #[serde(default)]
    pub demo_data: bool,
}

// Default value functions
fn default_description() -> String {
    "Custom Odoo addons".to_string()
}
fn default_license() -> String {
    "LGPL-3".to_string()
}
fn default_odoo_version() -> u8 {
    18
}
fn default_http_port() -> u16 {
    8069
}
fn default_longpolling_port() -> u16 {
    8072
}
fn default_admin_email() -> String {
    "admin".to_string()
}
fn default_admin_password() -> String {
    "admin".to_string()
}
fn default_postgresql_version() -> String {
    "17".to_string()
}
fn default_postgresql_port() -> u16 {
    5432
}
fn default_database_name() -> String {
    "firestream_odoo".to_string()
}
fn default_database_user() -> String {
    "firestream".to_string()
}
fn default_database_password() -> String {
    "firestream".to_string()
}
fn default_firestream_ref() -> String {
    "github:Cogent-Creation-Co/Firestream/nightly".to_string()
}

impl Default for OdooConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            description: default_description(),
            organization: String::new(),
            author: String::new(),
            author_email: String::new(),
            license: default_license(),
            odoo_version: default_odoo_version(),
            odoo_http_port: default_http_port(),
            odoo_longpolling_port: default_longpolling_port(),
            odoo_admin_email: default_admin_email(),
            odoo_admin_password: default_admin_password(),
            postgresql_version: default_postgresql_version(),
            postgresql_port: default_postgresql_port(),
            database_name: default_database_name(),
            database_user: default_database_user(),
            database_password: default_database_password(),
            firestream_ref: default_firestream_ref(),
            demo_data: false,
        }
    }
}

impl TemplateConfig for OdooConfig {
    fn project_name(&self) -> &str {
        &self.project_name
    }

    fn validate(&self) -> Result<()> {
        if self.project_name.is_empty() {
            return Err(TemplatizerError::invalid_config(
                "project_name cannot be empty",
            ));
        }

        if ![15, 16, 17, 18].contains(&self.odoo_version) {
            return Err(TemplatizerError::invalid_config(format!(
                "odoo_version must be 15, 16, 17, or 18 (got {})",
                self.odoo_version
            )));
        }

        if self.odoo_http_port == 0 {
            return Err(TemplatizerError::invalid_config(
                "odoo_http_port must be > 0",
            ));
        }

        if self.odoo_longpolling_port == 0 {
            return Err(TemplatizerError::invalid_config(
                "odoo_longpolling_port must be > 0",
            ));
        }

        if self.postgresql_port == 0 {
            return Err(TemplatizerError::invalid_config(
                "postgresql_port must be > 0",
            ));
        }

        Ok(())
    }

    fn to_context(&self) -> Result<Context> {
        // Start with default serde serialization
        let value = serde_json::to_value(self)?;
        let mut context = Context::from_value(value)?;

        // Inject derived fields
        let slug = to_kebab_case(&self.project_name);
        context.insert("project_slug", &slug);
        context.insert(
            "odoo_version_full",
            &format!("{}.0", self.odoo_version),
        );
        context.insert(
            "python_version",
            python_for_odoo_version(self.odoo_version),
        );

        Ok(context)
    }
}

/// Simple kebab-case conversion for project_slug derivation
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if prev_lower || (i > 0 && !result.ends_with('-')) {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_lower = false;
        } else if c == '_' || c == ' ' {
            result.push('-');
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }

    result
}

/// Builder for OdooConfig
#[derive(Debug, Clone, Default)]
pub struct OdooConfigBuilder {
    config: OdooConfig,
}

impl OdooConfigBuilder {
    /// Create a new builder with the required project name
    pub fn new(project_name: &str) -> Self {
        let mut config = OdooConfig::default();
        config.project_name = project_name.to_string();
        Self { config }
    }

    /// Set the Odoo major version
    pub fn odoo_version(mut self, version: u8) -> Self {
        self.config.odoo_version = version;
        self
    }

    /// Set the project description
    pub fn description(mut self, desc: &str) -> Self {
        self.config.description = desc.to_string();
        self
    }

    /// Set the organization
    pub fn organization(mut self, org: &str) -> Self {
        self.config.organization = org.to_string();
        self
    }

    /// Set the author
    pub fn author(mut self, author: &str) -> Self {
        self.config.author = author.to_string();
        self
    }

    /// Set the author email
    pub fn author_email(mut self, email: &str) -> Self {
        self.config.author_email = email.to_string();
        self
    }

    /// Set the Firestream flake reference
    pub fn firestream_ref(mut self, ref_url: &str) -> Self {
        self.config.firestream_ref = ref_url.to_string();
        self
    }

    /// Set the HTTP port
    pub fn http_port(mut self, port: u16) -> Self {
        self.config.odoo_http_port = port;
        self
    }

    /// Set the PostgreSQL version
    pub fn postgresql_version(mut self, version: &str) -> Self {
        self.config.postgresql_version = version.to_string();
        self
    }

    /// Set the database name
    pub fn database_name(mut self, name: &str) -> Self {
        self.config.database_name = name.to_string();
        self
    }

    /// Enable demo data
    pub fn demo_data(mut self, enabled: bool) -> Self {
        self.config.demo_data = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<OdooConfig> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build without validation (for testing)
    pub fn build_unchecked(self) -> OdooConfig {
        self.config
    }
}

/// Get an example OdooConfig matching example_context.json
pub fn get_example() -> OdooConfig {
    OdooConfigBuilder::new("Acme Odoo Addons")
        .odoo_version(18)
        .description("Custom Odoo modules for Acme Corporation")
        .organization("Acme Corp")
        .author("Acme Developer")
        .author_email("dev@acme.com")
        .build_unchecked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_valid() {
        let config = OdooConfigBuilder::new("Test Project")
            .odoo_version(18)
            .build();
        assert!(config.is_ok());
    }

    #[test]
    fn test_builder_empty_name_rejected() {
        let config = OdooConfigBuilder::new("")
            .odoo_version(18)
            .build();
        assert!(config.is_err());
    }

    #[test]
    fn test_invalid_version_rejected() {
        let config = OdooConfigBuilder::new("Test")
            .odoo_version(14)
            .build();
        assert!(config.is_err());

        let config = OdooConfigBuilder::new("Test")
            .odoo_version(19)
            .build();
        assert!(config.is_err());
    }

    #[test]
    fn test_python_version_mapping() {
        assert_eq!(python_for_odoo_version(15), "3.10");
        assert_eq!(python_for_odoo_version(16), "3.10");
        assert_eq!(python_for_odoo_version(17), "3.11");
        assert_eq!(python_for_odoo_version(18), "3.12");
    }

    #[test]
    fn test_derived_fields_in_context() {
        let config = OdooConfigBuilder::new("My Test Project")
            .odoo_version(17)
            .build()
            .unwrap();
        let context = config.to_context().unwrap();

        // Context.get returns Option<&Value>
        assert_eq!(
            context.get("project_slug").unwrap().as_str().unwrap(),
            "my-test-project"
        );
        assert_eq!(
            context.get("odoo_version_full").unwrap().as_str().unwrap(),
            "17.0"
        );
        assert_eq!(
            context.get("python_version").unwrap().as_str().unwrap(),
            "3.11"
        );
    }

    #[test]
    fn test_example_config_valid() {
        let config = get_example();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serialization_roundtrip_json() {
        let config = get_example();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: OdooConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.project_name, config.project_name);
        assert_eq!(parsed.odoo_version, config.odoo_version);
    }

    #[test]
    fn test_serialization_roundtrip_yaml() {
        let config = get_example();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: OdooConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.project_name, config.project_name);
        assert_eq!(parsed.odoo_version, config.odoo_version);
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(to_kebab_case("Hello World"), "hello-world");
        assert_eq!(to_kebab_case("my_project"), "my-project");
        assert_eq!(to_kebab_case("MyProject"), "my-project");
        assert_eq!(to_kebab_case("acme-odoo"), "acme-odoo");
    }
}
