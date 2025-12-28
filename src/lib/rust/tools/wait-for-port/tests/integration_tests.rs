use std::net::TcpListener;
use std::time::Duration;

use wait_for_port::{check_port_state, PortState, PortWaiter, WaitForPortError};

/// Find a free port for testing
fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to ephemeral port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

#[test]
fn test_check_port_state_free() {
    let port = find_free_port();

    // Port should be free since we just found it and didn't keep it bound
    let state = check_port_state("localhost", port).expect("Failed to check port state");
    assert_eq!(state, PortState::Free);
}

#[test]
fn test_check_port_state_inuse() {
    // Bind to a port and keep it bound
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();

    // Port should be in use since we're still holding the listener
    let state = check_port_state("localhost", port).expect("Failed to check port state");
    assert_eq!(state, PortState::InUse);

    // Drop the listener
    drop(listener);

    // Now port should be free
    let state = check_port_state("localhost", port).expect("Failed to check port state");
    assert_eq!(state, PortState::Free);
}

#[tokio::test]
async fn test_wait_for_port_already_inuse() {
    // Bind to a port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();

    // Wait should succeed immediately since port is already in use
    let result = PortWaiter::new(port)
        .host("localhost")
        .state(PortState::InUse)
        .timeout(Duration::from_secs(1))
        .wait()
        .await;

    assert!(result.is_ok());

    drop(listener);
}

#[tokio::test]
async fn test_wait_for_port_already_free() {
    let port = find_free_port();

    // Wait should succeed immediately since port is already free
    let result = PortWaiter::new(port)
        .host("localhost")
        .state(PortState::Free)
        .timeout(Duration::from_secs(1))
        .wait()
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_for_port_timeout() {
    // Bind to a port to keep it in use
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();

    // Wait for port to be free (should timeout since we're holding it)
    let result = PortWaiter::new(port)
        .host("localhost")
        .state(PortState::Free)
        .timeout(Duration::from_millis(500))
        .poll_interval(Duration::from_millis(100))
        .wait()
        .await;

    assert!(matches!(result, Err(WaitForPortError::Timeout { .. })));

    drop(listener);
}

#[tokio::test]
async fn test_wait_for_port_becomes_inuse() {
    let port = find_free_port();

    // Spawn a task that binds to the port after a delay
    let port_clone = port;
    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        TcpListener::bind(format!("127.0.0.1:{}", port_clone))
    });

    // Wait for port to become in use
    let result = PortWaiter::new(port)
        .host("localhost")
        .state(PortState::InUse)
        .timeout(Duration::from_secs(2))
        .poll_interval(Duration::from_millis(50))
        .wait()
        .await;

    assert!(result.is_ok());

    // Clean up
    let _ = handle.await;
}

#[tokio::test]
async fn test_wait_for_port_becomes_free() {
    // Bind to a port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();

    // Spawn a task that drops the listener after a delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(listener);
    });

    // Wait for port to become free
    let result = PortWaiter::new(port)
        .host("127.0.0.1")
        .state(PortState::Free)
        .timeout(Duration::from_secs(2))
        .poll_interval(Duration::from_millis(50))
        .wait()
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_port() {
    let result = PortWaiter::new(0)
        .host("localhost")
        .state(PortState::InUse)
        .timeout(Duration::from_secs(1))
        .wait()
        .await;

    assert!(matches!(result, Err(WaitForPortError::InvalidPort(0))));
}

#[test]
fn test_port_state_display() {
    assert_eq!(PortState::InUse.to_string(), "inuse");
    assert_eq!(PortState::Free.to_string(), "free");
}

#[test]
fn test_port_state_from_str() {
    assert_eq!(PortState::from_str("inuse"), Some(PortState::InUse));
    assert_eq!(PortState::from_str("INUSE"), Some(PortState::InUse));
    assert_eq!(PortState::from_str("free"), Some(PortState::Free));
    assert_eq!(PortState::from_str("FREE"), Some(PortState::Free));
    assert_eq!(PortState::from_str("invalid"), None);
}
