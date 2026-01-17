//! Goss YAML generation
//!
//! Generates goss.yaml files from specifications and templates.

use crate::spec::{
    goss::{
        CommandTest, FileTest, GossTests, HttpTest, PackageTest, PortTest, ProcessTest,
        ServiceTest,
    },
    GossSpec,
};
use std::collections::HashMap;

/// Goss YAML generator
pub struct GossYamlGenerator {
    /// Template engine
    template_engine: super::TemplateEngine,
}

impl GossYamlGenerator {
    /// Create a new Goss YAML generator
    pub fn new() -> Result<Self, super::Error> {
        // Load templates from the templates directory
        let templates_dir = Self::get_templates_dir()?;
        let template_engine = super::TemplateEngine::from_dir(&templates_dir)?;

        Ok(Self { template_engine })
    }

    /// Get the templates directory path
    fn get_templates_dir() -> Result<String, super::Error> {
        // First try relative to the current executable
        let exe_path = std::env::current_exe()
            .map_err(|e| super::Error::Io(e))?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| super::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot determine executable directory",
            )))?;

        let relative_templates = exe_dir.join("../templates");
        if relative_templates.exists() {
            return Ok(relative_templates.to_string_lossy().to_string());
        }

        // Try relative to current directory (for development)
        let current_dir = std::env::current_dir()
            .map_err(|e| super::Error::Io(e))?;
        let dev_templates = current_dir.join("templates");
        if dev_templates.exists() {
            return Ok(dev_templates.to_string_lossy().to_string());
        }

        // Try in the source tree (for cargo run)
        let src_templates = current_dir.join("src/lib/rust/tools/firestream-vib/templates");
        if src_templates.exists() {
            return Ok(src_templates.to_string_lossy().to_string());
        }

        Err(super::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Templates directory not found",
        )))
    }

    /// Generate goss.yaml from a spec
    pub fn generate(&self, spec: &GossSpec) -> Result<String, super::Error> {
        // If spec has a custom template, use it
        if let Some(template_path) = &spec.template {
            return self.generate_from_template(template_path, &spec.vars);
        }

        // Otherwise, generate from inline tests
        if let Some(tests) = &spec.tests {
            return self.generate_from_tests(tests, &spec.vars);
        }

        Err(super::Error::RenderFailed(
            "GossSpec must have either a template path or inline tests".to_string(),
        ))
    }

    /// Generate goss.yaml from inline test definitions
    fn generate_from_tests(
        &self,
        tests: &GossTests,
        vars: &HashMap<String, serde_json::Value>,
    ) -> Result<String, super::Error> {
        let mut sections = Vec::new();

        // File tests
        if !tests.file.is_empty() {
            let file_yaml = self.render_file_tests(&tests.file)?;
            sections.push(file_yaml);
        }

        // Package tests
        if !tests.package.is_empty() {
            let package_yaml = self.render_package_tests(&tests.package)?;
            sections.push(package_yaml);
        }

        // Service tests
        if !tests.service.is_empty() {
            let service_yaml = self.render_service_tests(&tests.service)?;
            sections.push(service_yaml);
        }

        // Port tests
        if !tests.port.is_empty() {
            let port_yaml = self.render_port_tests(&tests.port)?;
            sections.push(port_yaml);
        }

        // Process tests
        if !tests.process.is_empty() {
            let process_yaml = self.render_process_tests(&tests.process)?;
            sections.push(process_yaml);
        }

        // Command tests
        if !tests.command.is_empty() {
            let command_yaml = self.render_command_tests(&tests.command)?;
            sections.push(command_yaml);
        }

        // HTTP tests
        if !tests.http.is_empty() {
            let http_yaml = self.render_http_tests(&tests.http)?;
            sections.push(http_yaml);
        }

        // Combine all sections
        let yaml = sections.join("\n\n");
        Ok(yaml)
    }

    /// Generate goss.yaml from a template file
    pub fn generate_from_template(
        &self,
        template_path: &str,
        vars: &HashMap<String, serde_json::Value>,
    ) -> Result<String, super::Error> {
        // Read the template file
        let content = std::fs::read_to_string(template_path)
            .map_err(|e| super::Error::Io(e))?;

        // Create a temporary template engine and render
        let mut temp_engine = super::TemplateEngine::new()?;
        temp_engine.render_str(&content, vars)
    }

    /// Render file tests section
    fn render_file_tests(
        &self,
        file_tests: &HashMap<String, FileTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("file:");

        for (path, test) in file_tests {
            yaml.push_str(&format!("\n  {}:", path));
            yaml.push_str(&format!("\n    exists: {}", test.exists));

            if let Some(mode) = &test.mode {
                yaml.push_str(&format!("\n    mode: \"{}\"", mode));
            }

            if let Some(owner) = &test.owner {
                yaml.push_str(&format!("\n    owner: {}", owner));
            }

            if let Some(group) = &test.group {
                yaml.push_str(&format!("\n    group: {}", group));
            }

            if let Some(contains) = &test.contains {
                yaml.push_str("\n    contains:");
                for pattern in contains {
                    yaml.push_str(&format!("\n    - \"{}\"", pattern));
                }
            }
        }

        Ok(yaml)
    }

    /// Render package tests section
    fn render_package_tests(
        &self,
        package_tests: &HashMap<String, PackageTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("package:");

        for (name, test) in package_tests {
            yaml.push_str(&format!("\n  {}:", name));
            yaml.push_str(&format!("\n    installed: {}", test.installed));

            if let Some(version) = &test.version {
                yaml.push_str(&format!("\n    version: {}", version));
            }
        }

        Ok(yaml)
    }

    /// Render service tests section
    fn render_service_tests(
        &self,
        service_tests: &HashMap<String, ServiceTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("service:");

        for (name, test) in service_tests {
            yaml.push_str(&format!("\n  {}:", name));

            if let Some(enabled) = test.enabled {
                yaml.push_str(&format!("\n    enabled: {}", enabled));
            }

            if let Some(running) = test.running {
                yaml.push_str(&format!("\n    running: {}", running));
            }
        }

        Ok(yaml)
    }

    /// Render port tests section
    fn render_port_tests(
        &self,
        port_tests: &HashMap<String, PortTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("port:");

        for (port, test) in port_tests {
            yaml.push_str(&format!("\n  {}:", port));
            yaml.push_str(&format!("\n    listening: {}", test.listening));

            if let Some(ips) = &test.ip {
                yaml.push_str("\n    ip:");
                for ip in ips {
                    yaml.push_str(&format!("\n    - \"{}\"", ip));
                }
            }
        }

        Ok(yaml)
    }

    /// Render process tests section
    fn render_process_tests(
        &self,
        process_tests: &HashMap<String, ProcessTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("process:");

        for (name, test) in process_tests {
            yaml.push_str(&format!("\n  {}:", name));
            yaml.push_str(&format!("\n    running: {}", test.running));
        }

        Ok(yaml)
    }

    /// Render command tests section
    fn render_command_tests(
        &self,
        command_tests: &HashMap<String, CommandTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("command:");

        for (name, test) in command_tests {
            yaml.push_str(&format!("\n  {}:", name));
            yaml.push_str(&format!("\n    exit-status: {}", test.exit_status));

            if let Some(stdout) = &test.stdout {
                yaml.push_str("\n    stdout:");
                for pattern in stdout {
                    yaml.push_str(&format!("\n    - \"{}\"", pattern));
                }
            }

            if let Some(stderr) = &test.stderr {
                yaml.push_str("\n    stderr:");
                for pattern in stderr {
                    yaml.push_str(&format!("\n    - \"{}\"", pattern));
                }
            }

            if let Some(timeout) = test.timeout {
                yaml.push_str(&format!("\n    timeout: {}", timeout));
            }
        }

        Ok(yaml)
    }

    /// Render HTTP tests section
    fn render_http_tests(
        &self,
        http_tests: &HashMap<String, HttpTest>,
    ) -> Result<String, super::Error> {
        let mut yaml = String::from("http:");

        for (url, test) in http_tests {
            yaml.push_str(&format!("\n  {}:", url));
            yaml.push_str(&format!("\n    status: {}", test.status));

            if let Some(headers) = &test.headers {
                yaml.push_str("\n    headers:");
                for (key, value) in headers {
                    yaml.push_str(&format!("\n      {}: \"{}\"", key, value));
                }
            }

            if let Some(body) = &test.body {
                yaml.push_str("\n    body:");
                for pattern in body {
                    yaml.push_str(&format!("\n    - \"{}\"", pattern));
                }
            }

            if let Some(timeout) = test.timeout {
                yaml.push_str(&format!("\n    timeout: {}", timeout));
            }
        }

        Ok(yaml)
    }

    /// Validate generated goss.yaml
    pub fn validate(&self, yaml: &str) -> Result<(), super::Error> {
        // Try to parse as YAML to ensure it's valid
        let _: serde_yaml::Value = serde_yaml::from_str(yaml)
            .map_err(|e| super::Error::RenderFailed(format!("Invalid YAML: {}", e)))?;

        // Additional Goss-specific validation could go here
        // For now, just checking that it's valid YAML is sufficient

        Ok(())
    }

    /// Render a template from the templates directory
    pub fn render_template(
        &self,
        template_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String, super::Error> {
        self.template_engine.render(template_name, context)
    }

    /// Generate goss.yaml from Nix metadata (convenience method)
    pub fn generate_from_nix_metadata(
        &self,
        metadata: &crate::nix::metadata::GossMetadata,
        root_dir: &str,
    ) -> Result<String, super::Error> {
        let mut sections = Vec::new();

        // Check binaries
        if !metadata.binaries.is_empty() {
            let mut context = HashMap::new();
            context.insert(
                "binaries".to_string(),
                serde_json::to_value(&metadata.binaries).unwrap(),
            );
            let rendered = self.render_template("check-binaries.yaml.tera", &context)?;
            sections.push(rendered);
        }

        // Check directories
        if !metadata.directories.is_empty() {
            let mut context = HashMap::new();
            context.insert(
                "directories".to_string(),
                serde_json::to_value(&metadata.directories).unwrap(),
            );
            let rendered = self.render_template("check-directories.yaml.tera", &context)?;
            sections.push(rendered);
        }

        // Check version
        let mut context = HashMap::new();
        context.insert(
            "version".to_string(),
            serde_json::to_value(&metadata.version).unwrap(),
        );
        let rendered = self.render_template("check-version.yaml.tera", &context)?;
        sections.push(rendered);

        // Check symlinks
        if metadata.check_symlinks {
            let mut context = HashMap::new();
            context.insert(
                "root_dir".to_string(),
                serde_json::Value::String(root_dir.to_string()),
            );
            let rendered = self.render_template("check-symlinks.yaml.tera", &context)?;
            sections.push(rendered);
        }

        // Check linked libraries
        if let Some(linked_libs) = &metadata.check_linked_libraries {
            if linked_libs.enabled {
                let mut context = HashMap::new();
                context.insert(
                    "linked_libraries".to_string(),
                    serde_json::to_value(linked_libs).unwrap(),
                );
                let rendered = self.render_template("check-linked-libraries.yaml.tera", &context)?;
                sections.push(rendered);
            }
        }

        // Check CA certs
        if metadata.check_ca_certs {
            let context = HashMap::new();
            let rendered = self.render_template("check-ca-certs.yaml.tera", &context)?;
            sections.push(rendered);
        }

        // Check SPDX
        if metadata.check_spdx {
            let mut context = HashMap::new();
            context.insert(
                "root_dir".to_string(),
                serde_json::Value::String(root_dir.to_string()),
            );
            let rendered = self.render_template("check-spdx.yaml.tera", &context)?;
            sections.push(rendered);
        }

        // Combine all sections
        let yaml = sections.join("\n\n");
        Ok(yaml)
    }
}

impl Default for GossYamlGenerator {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
