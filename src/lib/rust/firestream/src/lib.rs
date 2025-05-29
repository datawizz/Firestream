//! Firestream CLI/TUI - Data Infrastructure Management Tool
//! 
//! This library provides both command-line and text-based user interfaces
//! for managing data infrastructure services like Kafka, PostgreSQL, etc.

pub mod cli;
pub mod config;
pub mod core;
pub mod deploy;
pub mod services;
pub mod state;
pub mod tui;

// Re-export commonly used types
pub use config::{GlobalConfig, ServiceConfig};
pub use core::error::{FirestreamError, Result};
pub use services::ServiceManager;
pub use state::StateManager;
