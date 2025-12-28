use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tracing::{debug, trace};

use crate::config::{PortState, WaitConfig};
use crate::error::{Result, WaitForPortError};

/// Check if a host is localhost
fn is_localhost(host: &str) -> bool {
    matches!(
        host.to_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1" | ""
    )
}

/// Check the current state of a port
///
/// Returns `PortState::InUse` if a service is listening on the port,
/// or `PortState::Free` if the port is available.
pub fn check_port_state(host: &str, port: u16) -> Result<PortState> {
    let addr = resolve_address(host, port)?;

    // Try to connect to the port
    match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
        Ok(_) => {
            trace!("Port {} on {} is in use (connection successful)", port, host);
            Ok(PortState::InUse)
        }
        Err(e) => {
            // Connection refused means port is free
            // For localhost, we can also try to bind to confirm
            if is_localhost(host) {
                check_localhost_port_free(port)
            } else {
                // For remote hosts, connection failure means port is likely free
                // (or host is unreachable, but we treat that as free for remote checks)
                trace!(
                    "Port {} on {} appears free (connection failed: {})",
                    port,
                    host,
                    e
                );
                Ok(PortState::Free)
            }
        }
    }
}

/// Check if a localhost port is free by attempting to bind to it
fn check_localhost_port_free(port: u16) -> Result<PortState> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    match TcpListener::bind(addr) {
        Ok(_listener) => {
            // We could bind, so the port is free
            // The listener is dropped here, releasing the port
            trace!("Port {} is free (bind successful)", port);
            Ok(PortState::Free)
        }
        Err(e) => {
            // Check if the error is "address already in use"
            if e.kind() == std::io::ErrorKind::AddrInUse {
                trace!("Port {} is in use (bind failed: address in use)", port);
                Ok(PortState::InUse)
            } else {
                // Some other error (permission denied, etc.)
                trace!("Port {} check failed: {}", port, e);
                Err(WaitForPortError::Io(e))
            }
        }
    }
}

/// Resolve a host:port to a SocketAddr
fn resolve_address(host: &str, port: u16) -> Result<SocketAddr> {
    let host = if host.is_empty() { "localhost" } else { host };
    let addr_str = format!("{}:{}", host, port);

    addr_str
        .to_socket_addrs()
        .map_err(|_| WaitForPortError::DnsResolutionFailed(host.to_string()))?
        .next()
        .ok_or_else(|| WaitForPortError::DnsResolutionFailed(host.to_string()))
}

/// Wait for a port to reach the desired state
///
/// Polls the port at regular intervals until it reaches the desired state
/// or the timeout is exceeded.
pub async fn wait_for_port(config: &WaitConfig) -> Result<()> {
    // Validate port
    if config.port == 0 {
        return Err(WaitForPortError::InvalidPort(0));
    }

    let deadline = Instant::now() + config.timeout;
    let timeout_secs = config.timeout.as_secs();

    debug!(
        "Waiting for port {} on {} to be {} (timeout: {}s)",
        config.port,
        config.host,
        config.state,
        timeout_secs
    );

    loop {
        // Check current port state
        let current_state = check_port_state(&config.host, config.port)?;

        if current_state == config.state {
            debug!(
                "Port {} on {} is now {}",
                config.port, config.host, config.state
            );
            return Ok(());
        }

        // Check if we've exceeded the deadline
        if Instant::now() >= deadline {
            return Err(WaitForPortError::Timeout {
                host: config.host.clone(),
                port: config.port,
                state: config.state.as_str().to_string(),
                timeout_secs,
            });
        }

        trace!(
            "Port {} on {} is currently {} (waiting for {})",
            config.port,
            config.host,
            current_state,
            config.state
        );

        // Wait before next check
        sleep(config.poll_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_localhost() {
        assert!(is_localhost("localhost"));
        assert!(is_localhost("LOCALHOST"));
        assert!(is_localhost("127.0.0.1"));
        assert!(is_localhost("::1"));
        assert!(is_localhost(""));
        assert!(!is_localhost("example.com"));
        assert!(!is_localhost("192.168.1.1"));
    }

    #[test]
    fn test_resolve_address() {
        let addr = resolve_address("localhost", 8080).unwrap();
        assert_eq!(addr.port(), 8080);

        let addr = resolve_address("127.0.0.1", 3000).unwrap();
        assert_eq!(addr.port(), 3000);
    }

    #[test]
    fn test_resolve_address_empty_host() {
        let addr = resolve_address("", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
    }
}
