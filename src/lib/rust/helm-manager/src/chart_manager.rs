use std::sync::Mutex;

use crate::{embedded_charts, ChartMetadata, Error, Result};

/// Manages chart operations
pub struct ChartManager {
    /// Embedded charts instance
    embedded_charts: Mutex<embedded_charts::EmbeddedCharts>,
}

impl ChartManager {
    /// Create a new ChartManager
    pub fn new() -> Self {
        Self {
            embedded_charts: Mutex::new(embedded_charts::EmbeddedCharts::new()),
        }
    }

    /// List available charts
    pub fn list_charts(&self) -> Vec<&str> {
        embedded_charts::list_available_charts()
    }

    /// Check if a chart exists
    pub fn has_chart(&self, name: &str) -> bool {
        self.embedded_charts
            .lock()
            .unwrap()
            .has_chart(name)
    }

    /// Get the path to a chart, extracting it if necessary
    pub async fn get_chart_path(&self, name: &str) -> Result<String> {
        let mut charts = self.embedded_charts
            .lock()
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
        
        let path = charts.extract_chart(name)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Get chart metadata
    pub async fn get_chart_metadata(&self, name: &str) -> Result<ChartMetadata> {
        let mut charts = self.embedded_charts
            .lock()
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
        
        charts.get_chart_metadata(name)
    }

    /// Get default values for a chart
    pub async fn get_default_values(&self, name: &str) -> Result<serde_json::Value> {
        let mut charts = self.embedded_charts
            .lock()
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to lock embedded charts: {}", e)))?;
        
        charts.get_default_values(name)
    }

    /// Validate a chart exists and has required files
    pub async fn validate_chart(&self, name: &str) -> Result<()> {
        if !self.has_chart(name) {
            return Err(Error::ChartNotFound(name.to_string()));
        }

        // Check that we can extract the chart
        self.get_chart_path(name).await?;

        // Check that Chart.yaml exists and is valid
        self.get_chart_metadata(name).await?;

        Ok(())
    }
}

impl Default for ChartManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_manager_creation() {
        let manager = ChartManager::new();
        let charts = manager.list_charts();
        assert!(!charts.is_empty());
    }

    #[tokio::test]
    async fn test_chart_validation() {
        let manager = ChartManager::new();
        
        // PostgreSQL should exist
        assert!(manager.validate_chart("postgresql").await.is_ok());
        
        // Non-existent chart should fail
        assert!(manager.validate_chart("nonexistent").await.is_err());
    }
}