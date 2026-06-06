//! Airflow chart implementation.
//!
//! Unlike the older hand-rolled charts in this module, `AirflowChart` is
//! driven entirely by the Nix-emitted `chart-manifest.json`. If the bundle
//! isn't available (no `FIRESTREAM_CHARTS_DIR` and no `/opt/firestream/charts`),
//! constructing this chart returns `None`. Callers should treat missing
//! manifests as "skip airflow" rather than as an error, because Wave 3 is
//! still wiring per-chart Nix modules and stack entries may legitimately
//! reference charts that aren't yet built.

use crate::deploy::helm_lifecycle::{
    from_manifest::chart_info_for, ChartInfo, CommonChart,
};

/// Airflow chart configuration sourced from the Nix-emitted manifest.
pub struct AirflowChart;

impl AirflowChart {
    /// The Firestream name under which the airflow chart is registered in
    /// `index.json`. Matches `firestreamCharts.airflow` in
    /// `nix/flake-modules/charts/airflow.nix`.
    pub const NAME: &'static str = "airflow";

    /// Construct an airflow chart from the manifest in the global registry.
    /// Returns `None` if the bundle isn't available or airflow isn't
    /// registered.
    pub fn from_registry() -> Option<CommonChart> {
        chart_info_for(Self::NAME).map(CommonChart::new)
    }

    /// Construct an airflow chart from an explicit `ChartInfo`. Used by the
    /// CLI when the chart name is resolved through `Charts::open` directly
    /// (so the caller can pass `--namespace` overrides).
    pub fn from_chart_info(chart_info: ChartInfo) -> CommonChart {
        CommonChart::new(chart_info)
    }
}
