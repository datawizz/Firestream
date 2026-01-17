//! wait-for-port: Wait for a TCP port to reach a desired state
//!
//! This crate provides functionality to wait for a TCP port to become available (inuse)
//! or free. It's useful for container orchestration and service startup sequencing.
//!
//! # Example
//!
//! ```no_run
//! use wait_for_port::{PortWaiter, PortState};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> wait_for_port::Result<()> {
//!     // Wait for a service to start on port 8080
//!     PortWaiter::new(8080)
//!         .host("localhost")
//!         .state(PortState::InUse)
//!         .timeout(Duration::from_secs(30))
//!         .wait()
//!         .await?;
//!
//!     println!("Service is ready!");
//!     Ok(())
//! }
//! ```

pub mod checker;
pub mod config;
pub mod error;

pub use checker::{check_port_state, wait_for_port};
pub use config::{PortState, WaitConfig, DEFAULT_POLL_INTERVAL, DEFAULT_TIMEOUT};
pub use error::{Result, WaitForPortError};

use std::time::Duration;

/// Builder for waiting on a port
///
/// Provides a fluent API for configuring and executing a port wait operation.
///
/// # Example
///
/// ```no_run
/// use wait_for_port::{PortWaiter, PortState};
/// use std::time::Duration;
///
/// # async fn example() -> wait_for_port::Result<()> {
/// // Wait for PostgreSQL to be ready
/// PortWaiter::new(5432)
///     .host("db.example.com")
///     .state(PortState::InUse)
///     .timeout(Duration::from_secs(60))
///     .wait()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct PortWaiter {
    config: WaitConfig,
}

impl PortWaiter {
    /// Create a new PortWaiter for the specified port
    ///
    /// Defaults:
    /// - host: "localhost"
    /// - state: InUse (waiting for service to start)
    /// - timeout: 30 seconds
    /// - poll_interval: 500ms
    pub fn new(port: u16) -> Self {
        Self {
            config: WaitConfig::new(port),
        }
    }

    /// Set the target host (IP address or hostname)
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Set the desired port state to wait for
    pub fn state(mut self, state: PortState) -> Self {
        self.config.state = state;
        self
    }

    /// Set the maximum time to wait
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the interval between port checks
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.config.poll_interval = interval;
        self
    }

    /// Wait for the port to reach the desired state
    ///
    /// Returns `Ok(())` if the port reaches the desired state within the timeout,
    /// or `Err(WaitForPortError::Timeout)` if the timeout is exceeded.
    pub async fn wait(self) -> Result<()> {
        wait_for_port(&self.config).await
    }

    /// Get a reference to the current configuration
    pub fn config(&self) -> &WaitConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_waiter_builder() {
        let waiter = PortWaiter::new(8080)
            .host("example.com")
            .state(PortState::Free)
            .timeout(Duration::from_secs(60))
            .poll_interval(Duration::from_millis(250));

        let config = waiter.config();
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 8080);
        assert_eq!(config.state, PortState::Free);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.poll_interval, Duration::from_millis(250));
    }

    #[test]
    fn test_port_waiter_defaults() {
        let waiter = PortWaiter::new(3000);
        let config = waiter.config();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3000);
        assert_eq!(config.state, PortState::InUse);
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert_eq!(config.poll_interval, DEFAULT_POLL_INTERVAL);
    }
}
