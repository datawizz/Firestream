//! Kubernetes provider implementations
//!
//! This module contains implementations for various Kubernetes providers.

// Local providers
pub mod k3d;  // K3D (K3s in Docker)

// Cloud providers - TODO: Implement these
pub mod gke;  // Google Kubernetes Engine
pub mod eks;  // Amazon Elastic Kubernetes Service
pub mod aks;  // Azure Kubernetes Service

// Re-export K3D provider components
pub use k3d::{K3dClusterManager, setup_cluster, setup_cluster_with_config, delete_cluster};
