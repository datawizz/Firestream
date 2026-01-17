//! # nix-container-builder
//!
//! A Rust library for building containers from Nix flakes.
//!
//! This crate provides a robust, type-safe API for building container images
//! from `flake.nix` files, supporting both native Nix builds on Linux and
//! Docker-based builds for cross-platform support.
//!
//! ## Features
//!
//! - **Platform Detection**: Automatically detects the best build strategy
//! - **Native Nix Builds**: Fast builds on Linux using native Nix
//! - **Docker-based Builds**: Cross-platform builds using Docker with Nix
//! - **Parallel Builds**: Build multiple containers concurrently
//! - **Progress Reporting**: Callbacks for build progress updates
//! - **Container Discovery**: Automatic discovery of available containers
//!
//! ## Example
//!
//! ```rust,no_run
//! use nix_container_builder::{NixContainerBuilder, BuildConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create builder with default config
//!     let builder = NixContainerBuilder::new().await?;
//!
//!     // List available containers
//!     let containers = builder.discover_containers().await?;
//!     for container in &containers {
//!         println!("Found container: {}", container.name);
//!     }
//!
//!     // Build a single container
//!     let result = builder.build("redis").await?;
//!     println!("Built: {}", result.full_ref());
//!
//!     // Build multiple containers in parallel
//!     let results = builder.build_parallel(
//!         &["redis", "postgres", "airflow"],
//!         Some(4), // max concurrent builds
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Build Strategies
//!
//! The library supports two build strategies:
//!
//! 1. **Native Nix** (`BuildStrategy::NativeNix`): Builds directly using the host's
//!    Nix installation. This is the fastest option but only works on Linux.
//!
//! 2. **Docker Nix** (`BuildStrategy::DockerNix`): Builds inside a Docker container
//!    with Nix. This works on all platforms including macOS.
//!
//! The library automatically selects the best strategy based on the platform,
//! but you can override this with `BuildConfig::force_native` or `BuildConfig::force_docker`.

pub mod builder;
pub mod config;
pub mod discovery;
pub mod docker_loader;
pub mod embedded;
pub mod error;
pub mod platform;
pub mod progress;
pub mod strategy;

// Re-export main types at crate root
pub use builder::{BuildResult, NixContainerBuilder};
pub use config::{BuildConfig, RetryConfig};
pub use discovery::ContainerInfo;
pub use docker_loader::{DockerLoader, LoadResult};
pub use embedded::{extract_embedded, ExtractedWorkspace};
pub use error::{NixContainerError, Result};
pub use platform::{Architecture, Platform, PlatformInfo};
pub use progress::{BuildPhase, BuildProgress, MultiProgress, ProgressCallback};
pub use strategy::{create_strategy_with_workspace, BuildStrategy, ContainerBuildStrategy};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::builder::{BuildResult, NixContainerBuilder};
    pub use crate::config::BuildConfig;
    pub use crate::discovery::ContainerInfo;
    pub use crate::error::{NixContainerError, Result};
    pub use crate::platform::PlatformInfo;
    pub use crate::progress::{BuildPhase, BuildProgress};
    pub use crate::strategy::BuildStrategy;
}
