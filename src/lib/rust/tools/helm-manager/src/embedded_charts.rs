use include_dir::Dir;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::{ChartMetadata, Error, Result};

// Include the generated manifest
include!(concat!(env!("OUT_DIR"), "/chart_manifest.rs"));

/// Embedded charts manager
pub struct EmbeddedCharts {
    /// Cache of extracted charts
    cache: HashMap<String, PathBuf>,
    /// Temporary directory for extracted charts
    temp_dir: Option<TempDir>,
}

impl EmbeddedCharts {
    /// Create a new embedded charts manager
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            temp_dir: None,
        }
    }

    /// List all available charts
    pub fn list_charts(&self) -> Vec<&'static str> {
        AVAILABLE_CHARTS.to_vec()
    }

    /// Check if a chart exists
    pub fn has_chart(&self, name: &str) -> bool {
        AVAILABLE_CHARTS.contains(&name)
    }

    /// Extract a chart to a temporary directory
    pub fn extract_chart(&mut self, name: &str) -> Result<PathBuf> {
        // Check if already cached
        if let Some(path) = self.cache.get(name) {
            return Ok(path.clone());
        }

        // Verify chart exists
        if !self.has_chart(name) {
            return Err(Error::ChartNotFound(name.to_string()));
        }

        // Create temp dir if needed
        if self.temp_dir.is_none() {
            self.temp_dir = Some(tempfile::tempdir()?);
        }

        let temp_dir = self.temp_dir.as_ref().unwrap();
        let chart_dir = temp_dir.path().join(name);

        // Extract chart files
        self.extract_dir(&CHARTS_DIR, name, &chart_dir)?;

        // Cache the path
        self.cache.insert(name.to_string(), chart_dir.clone());

        Ok(chart_dir)
    }

    /// Extract a directory from embedded data
    fn extract_dir(&self, dir: &Dir<'_>, chart_name: &str, target: &PathBuf) -> Result<()> {
        // Find the chart directory
        let chart_dir = dir.get_dir(chart_name)
            .ok_or_else(|| Error::ChartNotFound(chart_name.to_string()))?;

        // Create target directory
        std::fs::create_dir_all(target)?;

        // Extract all files
        self.extract_dir_recursive(chart_dir, target)?;

        Ok(())
    }

    /// Recursively extract directory contents
    fn extract_dir_recursive(&self, dir: &Dir<'_>, target: &PathBuf) -> Result<()> {
        // Extract files
        for file in dir.files() {
            let file_path = target.join(file.path().file_name().unwrap());
            std::fs::write(&file_path, file.contents())?;
        }

        // Extract subdirectories
        for subdir in dir.dirs() {
            let subdir_name = subdir.path().file_name().unwrap();
            let subdir_path = target.join(subdir_name);
            std::fs::create_dir_all(&subdir_path)?;
            self.extract_dir_recursive(subdir, &subdir_path)?;
        }

        Ok(())
    }

    /// Get chart metadata
    pub fn get_chart_metadata(&mut self, name: &str) -> Result<ChartMetadata> {
        let chart_path = self.extract_chart(name)?;
        let chart_yaml_path = chart_path.join("Chart.yaml");

        let content = std::fs::read_to_string(chart_yaml_path)?;
        let metadata: ChartMetadata = serde_yaml::from_str(&content)?;

        Ok(metadata)
    }

    /// Get default values for a chart
    pub fn get_default_values(&mut self, name: &str) -> Result<serde_json::Value> {
        let chart_path = self.extract_chart(name)?;
        let values_yaml_path = chart_path.join("values.yaml");

        if !values_yaml_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let content = std::fs::read_to_string(values_yaml_path)?;
        let values: serde_json::Value = serde_yaml::from_str(&content)?;

        Ok(values)
    }

    /// Get the path to an extracted chart
    pub fn get_chart_path(&mut self, name: &str) -> Result<PathBuf> {
        self.extract_chart(name)
    }
}

impl Default for EmbeddedCharts {
    fn default() -> Self {
        Self::new()
    }
}

/// Static instance of embedded charts
pub static EMBEDDED_CHARTS: Lazy<std::sync::Mutex<EmbeddedCharts>> = 
    Lazy::new(|| std::sync::Mutex::new(EmbeddedCharts::new()));

/// Get a list of available charts
pub fn list_available_charts() -> Vec<&'static str> {
    AVAILABLE_CHARTS.to_vec()
}

/// Extract a chart and return its path
pub fn extract_chart(name: &str) -> Result<PathBuf> {
    let mut charts = EMBEDDED_CHARTS.lock()
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
    charts.extract_chart(name)
}

/// Get chart metadata
pub fn get_chart_metadata(name: &str) -> Result<ChartMetadata> {
    let mut charts = EMBEDDED_CHARTS.lock()
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
    charts.get_chart_metadata(name)
}

/// Get default values for a chart
pub fn get_default_values(name: &str) -> Result<serde_json::Value> {
    let mut charts = EMBEDDED_CHARTS.lock()
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
    charts.get_default_values(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_charts() {
        let charts = list_available_charts();
        assert!(!charts.is_empty());
    }

    #[test]
    fn test_has_chart() {
        let embedded_charts = EmbeddedCharts::new();
        // At least PostgreSQL should be available
        assert!(embedded_charts.has_chart("postgresql"));
        assert!(!embedded_charts.has_chart("nonexistent"));
    }
}