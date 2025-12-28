use std::time::Duration;

/// The desired state of the port to wait for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    /// Port is in use (a service is listening)
    InUse,
    /// Port is free (available for binding)
    Free,
}

impl PortState {
    /// Parse from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "inuse" | "in_use" | "in-use" => Some(PortState::InUse),
            "free" | "available" => Some(PortState::Free),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PortState::InUse => "inuse",
            PortState::Free => "free",
        }
    }
}

impl std::fmt::Display for PortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Default poll interval (500ms, matching Bitnami behavior)
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Default timeout (30 seconds, matching Bitnami behavior)
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for waiting on a port
#[derive(Debug, Clone)]
pub struct WaitConfig {
    /// Target host (IP address or hostname)
    pub host: String,
    /// Port number to check
    pub port: u16,
    /// Desired port state
    pub state: PortState,
    /// Maximum time to wait
    pub timeout: Duration,
    /// Interval between checks
    pub poll_interval: Duration,
}

impl WaitConfig {
    /// Create a new WaitConfig with default values
    pub fn new(port: u16) -> Self {
        Self {
            host: "localhost".to_string(),
            port,
            state: PortState::InUse,
            timeout: DEFAULT_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }

    /// Set the target host
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the desired port state
    pub fn state(mut self, state: PortState) -> Self {
        self.state = state;
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the poll interval
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_state_from_str() {
        assert_eq!(PortState::from_str("inuse"), Some(PortState::InUse));
        assert_eq!(PortState::from_str("INUSE"), Some(PortState::InUse));
        assert_eq!(PortState::from_str("in_use"), Some(PortState::InUse));
        assert_eq!(PortState::from_str("in-use"), Some(PortState::InUse));
        assert_eq!(PortState::from_str("free"), Some(PortState::Free));
        assert_eq!(PortState::from_str("FREE"), Some(PortState::Free));
        assert_eq!(PortState::from_str("available"), Some(PortState::Free));
        assert_eq!(PortState::from_str("invalid"), None);
    }

    #[test]
    fn test_wait_config_builder() {
        let config = WaitConfig::new(8080)
            .host("example.com")
            .state(PortState::Free)
            .timeout(Duration::from_secs(60));

        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 8080);
        assert_eq!(config.state, PortState::Free);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }
}
