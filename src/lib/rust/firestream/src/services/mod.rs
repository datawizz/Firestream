//! Services module for managing data infrastructure services
//!
//! This module provides functionality for installing, starting, stopping,
//! and monitoring services.

pub mod manager;
pub mod kubernetes;

pub use manager::ServiceManager;
