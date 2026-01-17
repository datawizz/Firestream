use std::collections::HashMap;
use crate::ChartMetadata;

/// Chart registry for managing available charts
pub struct ChartRegistry {
    charts: HashMap<String, ChartMetadata>,
}

impl ChartRegistry {
    /// Create a new chart registry
    pub fn new() -> Self {
        Self {
            charts: HashMap::new(),
        }
    }

    /// Register a chart
    pub fn register(&mut self, metadata: ChartMetadata) {
        self.charts.insert(metadata.name.clone(), metadata);
    }

    /// Get chart metadata
    pub fn get(&self, name: &str) -> Option<&ChartMetadata> {
        self.charts.get(name)
    }

    /// List all charts
    pub fn list(&self) -> Vec<&str> {
        self.charts.keys().map(|k| k.as_str()).collect()
    }

    /// Search charts by keyword
    pub fn search(&self, keyword: &str) -> Vec<&ChartMetadata> {
        let keyword_lower = keyword.to_lowercase();
        
        self.charts.values()
            .filter(|chart| {
                chart.name.to_lowercase().contains(&keyword_lower) ||
                chart.description.as_ref()
                    .map(|d| d.to_lowercase().contains(&keyword_lower))
                    .unwrap_or(false) ||
                chart.keywords.as_ref()
                    .map(|keywords| keywords.iter().any(|k| k.to_lowercase().contains(&keyword_lower)))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get charts by category
    pub fn get_by_category(&self, category: &str) -> Vec<&ChartMetadata> {
        self.charts.values()
            .filter(|chart| {
                chart.keywords.as_ref()
                    .map(|keywords| keywords.iter().any(|k| k == category))
                    .unwrap_or(false)
            })
            .collect()
    }
}

impl Default for ChartRegistry {
    fn default() -> Self {
        Self::new()
    }
}