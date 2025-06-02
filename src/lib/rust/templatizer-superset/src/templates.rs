//! Tera templates for generating Superset YAML files

use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        
        // Register all templates from files
        tera.add_raw_template("metadata.yaml", include_str!("templates/metadata.yaml.tera"))
            .expect("Failed to load metadata template");
        tera.add_raw_template("database.yaml", include_str!("templates/database.yaml.tera"))
            .expect("Failed to load database template");
        tera.add_raw_template("dataset.yaml", include_str!("templates/dataset.yaml.tera"))
            .expect("Failed to load dataset template");
        tera.add_raw_template("chart.yaml", include_str!("templates/chart.yaml.tera"))
            .expect("Failed to load chart template");
        tera.add_raw_template("dashboard.yaml", include_str!("templates/dashboard.yaml.tera"))
            .expect("Failed to load dashboard template");
        
        tera
    };
}

/// Helper function to get time grain format for Superset
pub fn format_time_grain(grain: &str) -> String {
    match grain.to_lowercase().as_str() {
        "second" => "PT1S",
        "minute" => "PT1M", 
        "hour" => "PT1H",
        "day" => "P1D",
        "week" => "P1W",
        "month" => "P1M",
        "quarter" => "P3M",
        "year" => "P1Y",
        _ => grain,
    }.to_string()
}

/// Helper function to map chart types
pub fn map_chart_type(chart_type: &str) -> String {
    match chart_type {
        "line" => "line",
        "bar" => "bar", 
        "pie" => "pie",
        "table" => "table",
        "big_number" | "kpi" => "big_number_total",
        "area" => "area",
        "scatter" => "scatter",
        "bubble" => "bubble",
        "heatmap" => "heatmap",
        "box_plot" => "box_plot",
        "sunburst" => "sunburst",
        "sankey" => "sankey",
        "word_cloud" => "word_cloud",
        "treemap" => "treemap",
        "map" => "deck_geojson",
        _ => chart_type,
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Context;

    #[test]
    fn test_metadata_template() {
        let mut context = Context::new();
        context.insert("timestamp", "2023-05-31T10:00:00Z");
        
        let result = TEMPLATES.render("metadata.yaml", &context).unwrap();
        assert!(result.contains("version: 1.0.0"));
        assert!(result.contains("type: Dashboard"));
    }

    #[test]
    fn test_time_grain_format() {
        assert_eq!(format_time_grain("month"), "P1M");
        assert_eq!(format_time_grain("day"), "P1D");
        assert_eq!(format_time_grain("custom"), "custom");
    }

    #[test]
    fn test_chart_type_mapping() {
        assert_eq!(map_chart_type("line"), "line");
        assert_eq!(map_chart_type("big_number"), "big_number_total");
        assert_eq!(map_chart_type("kpi"), "big_number_total");
    }
}
