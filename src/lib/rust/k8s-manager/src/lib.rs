//! Kubernetes cluster management library
//!
//! This library provides a unified interface for managing various Kubernetes
//! cluster types including local (k3d), cloud providers (GKE, EKS, AKS), and more.

pub mod config;
pub mod error;
pub mod providers;
pub mod traits;

// Re-export commonly used types
pub use config::*;
pub use error::*;
pub use traits::*;

// Re-export provider implementations
pub use providers::k3d::{K3dClusterManager, setup_cluster, setup_cluster_with_config, delete_cluster};

// For backward compatibility, create a k3d module alias
pub mod k3d {
    pub use crate::providers::k3d::*;
}
