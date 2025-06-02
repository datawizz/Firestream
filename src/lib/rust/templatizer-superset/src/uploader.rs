//! Superset API upload functionality

use anyhow::{Context, Result, bail};
use reqwest::{Client, multipart};
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, info};

/// Superset API client for uploading dashboards
pub struct SupersetUploader {
    client: Client,
    base_url: String,
    username: String,
    password: String,
    access_token: Option<String>,
    csrf_token: Option<String>,
}

/// Login response from Superset
#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: String,
}

/// CSRF token response
#[derive(Debug, Deserialize)]
struct CsrfResponse {
    result: String,
}

/// Import response
#[derive(Debug, Deserialize)]
struct ImportResponse {
    message: String,
}

impl SupersetUploader {
    /// Create uploader from environment variables
    pub fn from_env(base_url: &str) -> Result<Self> {
        let username = std::env::var("SUPERSET_USERNAME")
            .context("SUPERSET_USERNAME environment variable not set")?;
        let password = std::env::var("SUPERSET_PASSWORD")
            .context("SUPERSET_PASSWORD environment variable not set")?;

        Self::new(base_url, username, password)
    }

    /// Create new uploader with credentials
    pub fn new(base_url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            username: username.into(),
            password: password.into(),
            access_token: None,
            csrf_token: None,
        })
    }

    /// Upload dashboard ZIP to Superset
    pub async fn upload_dashboard(&mut self, zip_data: Vec<u8>) -> Result<()> {
        // Authenticate if needed
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        // Get CSRF token if needed
        if self.csrf_token.is_none() {
            self.get_csrf_token().await?;
        }

        // Upload dashboard
        self.import_dashboard(zip_data).await?;

        Ok(())
    }

    /// Authenticate with Superset
    async fn authenticate(&mut self) -> Result<()> {
        info!("Authenticating with Superset");

        let url = format!("{}/api/v1/security/login", self.base_url);
        let payload = serde_json::json!({
            "username": self.username,
            "password": self.password,
            "provider": "db",
            "refresh": true
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send login request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Login failed with status {}: {}", status, text);
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .context("Failed to parse login response")?;

        self.access_token = Some(login_response.access_token);
        debug!("Successfully authenticated with Superset");

        Ok(())
    }

    /// Get CSRF token
    async fn get_csrf_token(&mut self) -> Result<()> {
        info!("Getting CSRF token");

        let access_token = self.access_token.as_ref()
            .context("No access token available")?;

        let url = format!("{}/api/v1/security/csrf_token/", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to get CSRF token")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Failed to get CSRF token with status {}: {}", status, text);
        }

        let csrf_response: CsrfResponse = response
            .json()
            .await
            .context("Failed to parse CSRF response")?;

        self.csrf_token = Some(csrf_response.result);
        debug!("Successfully obtained CSRF token");

        Ok(())
    }

    /// Import dashboard
    async fn import_dashboard(&self, zip_data: Vec<u8>) -> Result<()> {
        info!("Importing dashboard to Superset");

        let access_token = self.access_token.as_ref()
            .context("No access token available")?;
        let csrf_token = self.csrf_token.as_ref()
            .context("No CSRF token available")?;

        let url = format!("{}/api/v1/dashboard/import/", self.base_url);

        // Create multipart form
        let part = multipart::Part::bytes(zip_data)
            .file_name("dashboard.zip")
            .mime_str("application/zip")?;

        let form = multipart::Form::new()
            .part("formData", part)
            .text("overwrite", "true");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("X-CSRFToken", csrf_token)
            .header("Referer", &self.base_url)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload dashboard")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Dashboard import failed with status {}: {}", status, text);
        }

        let import_response: ImportResponse = response
            .json()
            .await
            .context("Failed to parse import response")?;

        info!("Dashboard imported successfully: {}", import_response.message);

        Ok(())
    }

    /// Upload dataset ZIP (if needed separately)
    pub async fn upload_dataset(&mut self, zip_data: Vec<u8>) -> Result<()> {
        // Authenticate if needed
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        // Get CSRF token if needed
        if self.csrf_token.is_none() {
            self.get_csrf_token().await?;
        }

        info!("Importing dataset to Superset");

        let access_token = self.access_token.as_ref()
            .context("No access token available")?;
        let csrf_token = self.csrf_token.as_ref()
            .context("No CSRF token available")?;

        let url = format!("{}/api/v1/dataset/import/", self.base_url);

        // Create multipart form
        let part = multipart::Part::bytes(zip_data)
            .file_name("dataset.zip")
            .mime_str("application/zip")?;

        let form = multipart::Form::new()
            .part("formData", part)
            .text("overwrite", "true");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("X-CSRFToken", csrf_token)
            .header("Referer", &self.base_url)
            .multipart(form)
            .send()
            .await
            .context("Failed to upload dataset")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Dataset import failed with status {}: {}", status, text);
        }

        info!("Dataset imported successfully");

        Ok(())
    }

    /// Test connection to Superset
    pub async fn test_connection(&mut self) -> Result<bool> {
        match self.authenticate().await {
            Ok(_) => {
                info!("Successfully connected to Superset");
                Ok(true)
            }
            Err(e) => {
                info!("Failed to connect to Superset: {}", e);
                Ok(false)
            }
        }
    }

    /// Get list of databases (for validation)
    pub async fn list_databases(&mut self) -> Result<Vec<String>> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let access_token = self.access_token.as_ref()
            .context("No access token available")?;

        let url = format!("{}/api/v1/database/", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Failed to list databases");
        }

        // Parse response and extract database names
        let data: serde_json::Value = response.json().await?;
        let databases = data["result"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|db| db["database_name"].as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(databases)
    }
}

/// Builder for SupersetUploader
pub struct UploaderBuilder {
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl UploaderBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            username: None,
            password: None,
        }
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn from_env(mut self) -> Result<Self> {
        self.username = Some(std::env::var("SUPERSET_USERNAME")
            .context("SUPERSET_USERNAME not set")?);
        self.password = Some(std::env::var("SUPERSET_PASSWORD")
            .context("SUPERSET_PASSWORD not set")?);
        Ok(self)
    }

    pub fn build(self) -> Result<SupersetUploader> {
        let username = self.username.context("Username not provided")?;
        let password = self.password.context("Password not provided")?;
        SupersetUploader::new(self.base_url, username, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploader_builder() {
        std::env::set_var("SUPERSET_USERNAME", "test_user");
        std::env::set_var("SUPERSET_PASSWORD", "test_pass");

        let uploader = UploaderBuilder::new("http://localhost:8088")
            .from_env()
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(uploader.base_url, "http://localhost:8088");
        assert_eq!(uploader.username, "test_user");
    }
}
