//! Configuration types for nix-container-builder

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Main configuration for building containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Path to containers directory (default: src/containers/firestream)
    pub containers_dir: PathBuf,

    /// Nix attribute to build (default: dockerImage)
    pub nix_attribute: String,

    /// Force native Nix build even if Docker available
    pub force_native: bool,

    /// Force Docker-based build even on Linux
    pub force_docker: bool,

    /// Named volume for Nix store persistence
    pub nix_store_volume: Option<String>,

    /// Build timeout in seconds
    #[serde(with = "duration_serde")]
    pub timeout: Duration,

    /// Retry configuration
    pub retries: RetryConfig,

    /// Maximum concurrent builds (for parallel building)
    pub max_concurrent: usize,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            containers_dir: PathBuf::from("src/containers/firestream"),
            nix_attribute: "dockerImage".to_string(),
            force_native: false,
            force_docker: false,
            nix_store_volume: None,
            timeout: Duration::from_secs(600),
            retries: RetryConfig::default(),
            max_concurrent: num_cpus::get(),
        }
    }
}

impl BuildConfig {
    /// Create a new BuildConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a BuildConfig optimized for embedded/portable mode
    ///
    /// This configuration forces Docker-based builds since the portable binary
    /// cannot rely on native Nix being installed on the host.
    pub fn for_embedded() -> Self {
        Self {
            // containers_dir is ignored in embedded mode (set by ExtractedWorkspace)
            containers_dir: PathBuf::from("embedded"),
            nix_attribute: "dockerImage".to_string(),
            force_native: false,
            force_docker: true, // Always use Docker in portable mode
            nix_store_volume: None,
            timeout: Duration::from_secs(600),
            retries: RetryConfig::default(),
            max_concurrent: num_cpus::get(),
        }
    }

    /// Set the containers directory
    pub fn with_containers_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.containers_dir = path.into();
        self
    }

    /// Set the Nix attribute to build
    pub fn with_nix_attribute(mut self, attr: impl Into<String>) -> Self {
        self.nix_attribute = attr.into();
        self
    }

    /// Force native Nix build
    pub fn with_force_native(mut self, force: bool) -> Self {
        self.force_native = force;
        self
    }

    /// Force Docker-based build
    pub fn with_force_docker(mut self, force: bool) -> Self {
        self.force_docker = force;
        self
    }

    /// Set the Nix store volume name
    pub fn with_nix_store_volume(mut self, volume: impl Into<String>) -> Self {
        self.nix_store_volume = Some(volume.into());
        self
    }

    /// Set the build timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the retry configuration
    pub fn with_retries(mut self, retries: RetryConfig) -> Self {
        self.retries = retries;
        self
    }

    /// Set maximum concurrent builds
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
}

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt using exponential backoff
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay_ms * 2u64.pow(attempt);
        Duration::from_millis(delay_ms.min(self.max_delay_ms))
    }
}

/// Serialization helpers for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BuildConfig::default();
        assert_eq!(config.nix_attribute, "dockerImage");
        assert!(!config.force_native);
        assert!(!config.force_docker);
        assert_eq!(config.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_builder_pattern() {
        let config = BuildConfig::new()
            .with_containers_dir("/custom/path")
            .with_nix_attribute("customAttr")
            .with_force_native(true)
            .with_timeout(Duration::from_secs(300));

        assert_eq!(config.containers_dir, PathBuf::from("/custom/path"));
        assert_eq!(config.nix_attribute, "customAttr");
        assert!(config.force_native);
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_retry_backoff() {
        let config = RetryConfig::default();

        // First retry: 1000ms
        assert_eq!(config.calculate_delay(0), Duration::from_millis(1000));

        // Second retry: 2000ms
        assert_eq!(config.calculate_delay(1), Duration::from_millis(2000));

        // Third retry: 4000ms
        assert_eq!(config.calculate_delay(2), Duration::from_millis(4000));

        // Should cap at max_delay_ms (10000)
        assert_eq!(config.calculate_delay(10), Duration::from_millis(10000));
    }
}
