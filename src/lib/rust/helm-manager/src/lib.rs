//! Helm Manager - Rust building blocks for invoking the Helm CLI.
//!
//! This crate exposes:
//!
//! - [`helm_client::HelmClient`] — a thin async wrapper around the `helm` binary
//!   (install / upgrade / uninstall / rollback / status / list).
//! - [`kubectl_client::KubectlClient`] — a matching wrapper around `kubectl`.
//! - [`values_resolver::resolve_values`] — merges values from files, env vars,
//!   and inline overrides into a single JSON value suitable for `--values`.
//! - [`Deployment`] / [`Stack`] builders and supporting data types
//!   ([`Release`], [`ReleaseStatus`], [`Values`], [`Chart`], [`ChartMetadata`])
//!   for describing what to deploy.
//!
//! Chart discovery and embedding used to live here. Both have moved to the
//! `firestream-charts` crate, which reads the flake-emitted chart index at
//! `/opt/firestream/charts`. This crate is now strictly the helm-CLI
//! execution layer; callers are expected to supply chart paths (or
//! repo/version coordinates) that helm can resolve on its own.
//!
//! # Example
//!
//! ```no_run
//! use helm_manager::helm_client::HelmClient;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), helm_manager::Error> {
//! let helm = HelmClient::new()?;
//! let release = helm
//!     .install(
//!         "my-database",
//!         "/opt/firestream/charts/postgresql",
//!         "default",
//!         json!({}),
//!         true,  // wait
//!         false, // atomic
//!     )
//!     .await?;
//! println!("Deployed: {}", release.name);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod deployment;
pub mod error;
pub mod helm_client;
pub mod kubectl_client;
pub mod models;
pub mod providers;
pub mod traits;
pub mod values_resolver;

// Re-export commonly used types
pub use config::Config;
pub use deployment::{Deployment, DeploymentBuilder, Stack, StackBuilder};
pub use error::{Error, Result};
pub use models::{Chart, ChartMetadata, Release, ReleaseStatus, Values};
