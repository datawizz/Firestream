//! Test result caching
//!
//! Caches test results based on Nix hashes to avoid redundant test runs.

pub mod nix_hash;

pub use nix_hash::NixHashCache;

/// Error type for cache operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cache miss for hash: {0}")]
    CacheMiss(String),

    #[error("Cache corruption: {0}")]
    Corruption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
