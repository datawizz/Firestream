//! K3D (K3s in Docker) provider implementation
//!
//! This module provides comprehensive K3D cluster management including:
//! - Basic cluster creation and deletion
//! - Advanced features with state integration
//! - Registry setup and management
//! - TLS certificate configuration
//! - Network configuration (routes, DNS)
//! - Development mode with port forwarding

// Core modules
pub mod manager;
pub mod operations;
pub mod utils;

// Feature modules
pub mod development;
pub mod networking;
pub mod observability;
pub mod registry;
pub mod security;

// Trait implementations
pub mod traits;

// Re-export main types and functions
pub use manager::K3dClusterManager;
pub use operations::{
    setup_cluster, 
    setup_cluster_with_config, 
    delete_cluster,
    check_k3d_installed,
};

// Re-export from feature modules if needed
pub use registry::create_registry;