//! External DNS chart implementation

use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartSetValue, HelmChartNamespace,
    values_path, CommonChart,
};

/// External DNS chart configuration
pub struct ExternalDnsChart;

impl ExternalDnsChart {
    /// Create External DNS chart with default configuration
    pub fn default() -> CommonChart {
        let chart_info = ChartInfo {
            name: "external-dns".to_string(),
            repository: Some("bitnami".to_string()),
            chart: "external-dns".to_string(),
            version: Some("6.31.0".to_string()),
            namespace: HelmChartNamespace::KubeSystem,
            values_files: vec![values_path("external-dns.yaml")],
            values: vec![
                // Resource limits
                ChartSetValue {
                    key: "resources.limits.cpu".to_string(),
                    value: "50m".to_string(),
                },
                ChartSetValue {
                    key: "resources.requests.cpu".to_string(),
                    value: "50m".to_string(),
                },
                ChartSetValue {
                    key: "resources.limits.memory".to_string(),
                    value: "100Mi".to_string(),
                },
                ChartSetValue {
                    key: "resources.requests.memory".to_string(),
                    value: "50Mi".to_string(),
                },
                // Provider configuration
                ChartSetValue {
                    key: "provider".to_string(),
                    value: "aws".to_string(),
                },
                ChartSetValue {
                    key: "policy".to_string(),
                    value: "sync".to_string(),
                },
                ChartSetValue {
                    key: "sources[0]".to_string(),
                    value: "service".to_string(),
                },
                ChartSetValue {
                    key: "sources[1]".to_string(),
                    value: "ingress".to_string(),
                },
            ],
            ..Default::default()
        };
        
        CommonChart::new(chart_info)
    }
    
    /// Create External DNS for local development (noop provider)
    pub fn local() -> CommonChart {
        let mut chart = Self::default();
        let info = chart.get_chart_info_mut();
        
        // Override provider for local development
        info.values.retain(|v| v.key != "provider");
        info.values.push(ChartSetValue {
            key: "provider".to_string(),
            value: "noop".to_string(),
        });
        
        // Add registry filter
        info.values.push(ChartSetValue {
            key: "registry".to_string(),
            value: "txt".to_string(),
        });
        
        info.values.push(ChartSetValue {
            key: "txtOwnerId".to_string(),
            value: "firestream-local".to_string(),
        });
        
        chart
    }
}
