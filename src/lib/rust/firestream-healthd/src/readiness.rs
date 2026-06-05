//! Readiness sub-check.
//!
//! `firestream-healthd` is generic across container runtime types. The
//! per-image readiness behaviour is configured at process start via flags;
//! this module owns the actual probe execution.
//!
//! Each [`ReadinessCheck`] runs a *single* attempt. The caller (typically a
//! compose / k8s readiness probe) is responsible for retries.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::Duration;

/// What kind of readiness check to run.
#[derive(Debug, Clone)]
pub enum ReadinessCheck {
    /// Always ready. No external dependency configured.
    None,
    /// Run a shell command. Ready iff `sh -c <cmd>` exits 0.
    Cmd(String),
    /// Open a TCP connection to `localhost:<port>`. Ready iff connect succeeds.
    Tcp(u16),
    /// HTTP GET. Ready iff connect succeeds and HTTP status < 500.
    Http(String),
}

/// Outcome of a single readiness attempt.
#[derive(Debug)]
pub enum ReadinessOutcome {
    Ready,
    NotReady(String),
}

impl ReadinessCheck {
    /// Run one attempt of the configured check. Never retries; never blocks
    /// for more than a few seconds (TCP/HTTP timeouts are short, command
    /// inherits the process's stdio settings).
    pub fn run(&self) -> ReadinessOutcome {
        match self {
            ReadinessCheck::None => ReadinessOutcome::Ready,
            ReadinessCheck::Cmd(cmd) => run_cmd(cmd),
            ReadinessCheck::Tcp(port) => run_tcp(*port),
            ReadinessCheck::Http(url) => run_http(url),
        }
    }

    /// Short, human-readable name for logs.
    pub fn name(&self) -> &'static str {
        match self {
            ReadinessCheck::None => "none",
            ReadinessCheck::Cmd(_) => "cmd",
            ReadinessCheck::Tcp(_) => "tcp",
            ReadinessCheck::Http(_) => "http",
        }
    }
}

fn run_cmd(cmd: &str) -> ReadinessOutcome {
    match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(out) if out.status.success() => ReadinessOutcome::Ready,
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stderr_tail = String::from_utf8_lossy(&out.stderr);
            let stderr_tail = stderr_tail.trim().lines().rev().next().unwrap_or("");
            ReadinessOutcome::NotReady(format!(
                "cmd exit {} (stderr tail: {})",
                code,
                stderr_tail.chars().take(120).collect::<String>()
            ))
        }
        Err(e) => ReadinessOutcome::NotReady(format!("cmd failed to spawn: {}", e)),
    }
}

fn run_tcp(port: u16) -> ReadinessOutcome {
    let addrs: Vec<SocketAddr> = match ("127.0.0.1", port).to_socket_addrs() {
        Ok(it) => it.collect(),
        Err(e) => return ReadinessOutcome::NotReady(format!("tcp resolve: {}", e)),
    };
    let Some(addr) = addrs.into_iter().next() else {
        return ReadinessOutcome::NotReady("tcp: no address resolved".into());
    };
    match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
        Ok(_) => ReadinessOutcome::Ready,
        Err(e) => ReadinessOutcome::NotReady(format!("tcp connect localhost:{}: {}", port, e)),
    }
}

/// Tiny HTTP/1.0 client. We deliberately don't pull in reqwest/hyper here —
/// `firestream-healthd` is supposed to stay slim, and the probe just needs
/// "did we get a status line < 500".
fn run_http(url: &str) -> ReadinessOutcome {
    // Accept only http://host[:port]/path
    let Some(rest) = url.strip_prefix("http://") else {
        return ReadinessOutcome::NotReady(format!("http: only http:// supported, got {}", url));
    };
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a.to_string(), format!("/{}", p)),
        None => (rest.to_string(), "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => match p.parse::<u16>() {
            Ok(n) => (h.to_string(), n),
            Err(_) => (authority.clone(), 80),
        },
        None => (authority.clone(), 80),
    };

    let addrs: Vec<SocketAddr> = match (host.as_str(), port).to_socket_addrs() {
        Ok(it) => it.collect(),
        Err(e) => return ReadinessOutcome::NotReady(format!("http resolve: {}", e)),
    };
    let Some(addr) = addrs.into_iter().next() else {
        return ReadinessOutcome::NotReady("http: no address resolved".into());
    };

    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
        Ok(s) => s,
        Err(e) => return ReadinessOutcome::NotReady(format!("http connect: {}", e)),
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let req = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nUser-Agent: firestream-healthd\r\nConnection: close\r\n\r\n",
        path, host
    );
    if let Err(e) = stream.write_all(req.as_bytes()) {
        return ReadinessOutcome::NotReady(format!("http write: {}", e));
    }

    // Read just enough to capture the status line.
    let mut buf = [0u8; 256];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(e) => return ReadinessOutcome::NotReady(format!("http read: {}", e)),
    };
    let head = String::from_utf8_lossy(&buf[..n]);
    let status_line = head.lines().next().unwrap_or("");
    // Expected: "HTTP/1.x NNN Reason"
    let code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if code > 0 && code < 500 {
        ReadinessOutcome::Ready
    } else {
        ReadinessOutcome::NotReady(format!("http status: {}", status_line))
    }
}
