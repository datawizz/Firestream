//! Nix hash-based caching
//!
//! Implements a cache keyed by Nix NAR hashes for test results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::runner::TestResult;

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Nix hash (cache key)
    pub hash: String,
    /// Timestamp of cache entry
    pub timestamp: String,
    /// Test results
    pub results: Vec<TestResult>,
}

/// Nix hash-based cache
pub struct NixHashCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// In-memory cache
    entries: HashMap<String, CacheEntry>,
}

impl NixHashCache {
    /// Create a new cache with a custom directory
    pub fn new(cache_dir: PathBuf) -> Result<Self, super::Error> {
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            cache_dir,
            entries: HashMap::new(),
        })
    }

    /// Create a cache in the default location
    pub fn default() -> Result<Self, super::Error> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| super::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine cache directory"
            )))?
            .join("firestream-vib");

        Self::new(cache_dir)
    }

    /// Get cached results for a hash
    pub fn get(&mut self, hash: &str) -> Result<Vec<TestResult>, super::Error> {
        // Check in-memory cache first
        if let Some(entry) = self.entries.get(hash) {
            return Ok(entry.results.clone());
        }

        // Load from disk
        let cache_file = self.cache_dir.join(format!("{}.json", hash));
        if !cache_file.exists() {
            return Err(super::Error::CacheMiss(hash.to_string()));
        }

        let content = std::fs::read_to_string(&cache_file)?;
        let entry: CacheEntry = serde_json::from_str(&content)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;

        // Update in-memory cache
        self.entries.insert(hash.to_string(), entry.clone());

        Ok(entry.results)
    }

    /// Store results in cache
    pub fn put(&mut self, hash: &str, results: Vec<TestResult>) -> Result<(), super::Error> {
        let entry = CacheEntry {
            hash: hash.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            results,
        };

        // Update in-memory cache
        self.entries.insert(hash.to_string(), entry.clone());

        // Write to disk
        let cache_file = self.cache_dir.join(format!("{}.json", hash));
        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;

        std::fs::write(cache_file, json)?;

        Ok(())
    }

    /// Check if results are cached
    pub fn contains(&self, hash: &str) -> bool {
        if self.entries.contains_key(hash) {
            return true;
        }

        let cache_file = self.cache_dir.join(format!("{}.json", hash));
        cache_file.exists()
    }

    /// Clear all cached results
    pub fn clear(&mut self) -> Result<(), super::Error> {
        self.entries.clear();

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                std::fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    /// Remove old cache entries (older than days)
    pub fn prune(&mut self, days: u64) -> Result<usize, super::Error> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let mut removed = 0;

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&cache_entry.timestamp) {
                    if timestamp.with_timezone(&chrono::Utc) < cutoff {
                        std::fs::remove_file(&path)?;
                        self.entries.remove(&cache_entry.hash);
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}
