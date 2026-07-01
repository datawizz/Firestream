//! Firestream - Data Infrastructure Management
//!
//! This is the main library crate that aggregates all Firestream functionality.
//! Use this crate as the single entry point for all infrastructure operations.
//!
//! ## Architecture
//!
//! Firestream provides a unified interface for managing data infrastructure:
//! - Kubernetes cluster management (k3d, cloud providers)
//! - Docker container operations
//! - Helm chart deployment
//! - Application templating (Spark, Puppeteer, Superset)
//! - Container verification
//!
//! ## Example
//!
//! ```rust,ignore
//! use firestream::{k8s_manager, docker_manager, helm_manager};
//! ```

pub mod app;
pub mod cli;
pub mod config;
pub mod core;
pub mod deploy;
pub mod services;
pub mod state;
pub mod template;
pub mod tui;

// Re-export infrastructure crates
// These provide the core infrastructure management capabilities
pub use docker_manager;
pub use filesystem_manager;
pub use firestream_vib;
pub use helm_manager;
pub use k8s_manager;
pub use templatizer;
pub use wait_for_port;

// Re-export commonly used types from internal modules
pub use config::{GlobalConfig, ServiceConfig};
pub use core::error::{FirestreamError, Result};
pub use services::ServiceManager;
pub use state::StateManager;
