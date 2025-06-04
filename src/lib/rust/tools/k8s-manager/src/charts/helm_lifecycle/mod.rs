//! Helm lifecycle management module
//!
//! This module provides a trait-based system for managing Helm chart lifecycles,
//! inspired by Qovery's approach but adapted for Firestream's architecture.

pub mod types;
pub mod trait_def;
pub mod lifecycle;
pub mod executor;
pub mod charts;
pub mod app;

pub use types::*;
pub use trait_def::*;
pub use lifecycle::*;
pub use executor::*;

// Re-export commonly used chart implementations
pub use charts::common::CommonChart;
pub use charts::prometheus::PrometheusOperatorChart;
pub use charts::external_dns::ExternalDnsChart;
pub use charts::nginx::NginxIngressChart;

// Re-export app module types
pub use app::{AppLifecycle, AppManifest, AppStructure};
