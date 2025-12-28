use thiserror::Error;

/// Errors that can occur while waiting for a port
#[derive(Error, Debug)]
pub enum WaitForPortError {
    #[error("Timeout after {timeout_secs}s waiting for port {port} on {host} to be {state}")]
    Timeout {
        host: String,
        port: u16,
        state: String,
        timeout_secs: u64,
    },

    #[error("Invalid port number: {0} (must be 1-65535)")]
    InvalidPort(u16),

    #[error("Cannot resolve host: {0}")]
    DnsResolutionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for wait-for-port operations
pub type Result<T> = std::result::Result<T, WaitForPortError>;
