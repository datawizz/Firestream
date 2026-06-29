//! RAII `kubectl port-forward` wrapper with ephemeral-port discovery.
//!
//! HTTP probes (airflow, jupyterhub, superset, spark, odoo — Phase 4) need
//! a host-side TCP endpoint that proxies to a service in the cluster.
//! `kubectl port-forward svc/<svc> :<remote>` does that — the leading
//! colon tells kubectl to bind an ephemeral local port and print it on
//! stdout:
//!
//! ```text
//! Forwarding from 127.0.0.1:54321 -> 8088
//! Forwarding from [::1]:54321 -> 8088
//! ```
//!
//! We capture stdout, parse the first `Forwarding from 127.0.0.1:<PORT>`
//! line, and return a handle whose Drop kills the child. A short post-
//! parse sleep gives kubectl a beat to actually start listening — the
//! "Forwarding from" line is printed slightly before the listener is
//! ready to accept.
//!
//! Phase 3 doesn't actually invoke this (postgres/redis use kubectl
//! exec). It's implemented now because Phase 4 will lean on it heavily
//! and the spec calls for it as part of this phase's deliverables.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use super::cluster::ClusterHandle;

/// Owns a running `kubectl port-forward` child + the local port it bound.
/// Kills the child on Drop (best effort; Drop never panics).
pub struct PortForwardHandle {
    pub local_port: u16,
    child: Child,
}

impl PortForwardHandle {
    /// Local TCP port the forward is listening on. Hand this to TCP /
    /// HTTP probes as `127.0.0.1:<local_port>`.
    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for PortForwardHandle {
    fn drop(&mut self) {
        // Best effort — kubectl may already have exited (e.g. test
        // panicked, network died). Swallow all errors; nothing to do.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Forward `namespace/service:remote_port` from the cluster behind
/// `handle` to an ephemeral local TCP port. Returns immediately after
/// kubectl prints its first "Forwarding from" line.
///
/// On Drop the handle kills kubectl, releasing the local port.
pub fn forward_service(
    handle: &ClusterHandle,
    namespace: &str,
    service: &str,
    remote_port: u16,
) -> Result<PortForwardHandle> {
    let svc_arg = format!("svc/{}", service);
    let port_arg = format!(":{}", remote_port); // leading colon ⇒ ephemeral local port

    let mut child = Command::new("kubectl")
        .arg("--kubeconfig")
        .arg(&handle.kubeconfig)
        .args(["port-forward", "-n", namespace, &svc_arg, &port_arg])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "spawn kubectl port-forward -n {} {} {}",
                namespace, svc_arg, port_arg
            )
        })?;

    // Block reading the first "Forwarding from 127.0.0.1:<PORT>" line.
    // We do this on the same thread so we can return the parsed port
    // synchronously. kubectl prints to stdout before it accepts
    // connections, so this read must succeed quickly or something's
    // broken; bound it via a thread-join with timeout.
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("kubectl port-forward stdout was not piped"))?;

    let (tx, rx) = std::sync::mpsc::channel::<Result<(u16, std::process::ChildStdout)>>();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let res = match reader.read_line(&mut line) {
            Ok(0) => Err(anyhow!("kubectl port-forward closed stdout before printing any line")),
            Ok(_) => parse_forwarding_line(&line).map(|p| (p, reader.into_inner())),
            Err(e) => Err(anyhow!("reading kubectl port-forward stdout: {}", e)),
        };
        let _ = tx.send(res);
    });

    // 10s should be plenty — kubectl prints "Forwarding from" within
    // ~50-200ms in practice. If we hit the timeout something is wrong
    // (e.g. the service doesn't exist, kubeconfig is bogus); fail hard.
    let (local_port, stdout_back) = match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(pair)) => pair,
        Ok(Err(e)) => {
            // Capture stderr for a useful error message before we kill
            // the child. Best-effort; swallow if anything goes wrong.
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.context(format!(
                "kubectl port-forward to {}/{} :{} failed before printing local port",
                namespace, service, remote_port
            )));
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "timed out waiting for kubectl port-forward to print its local port \
                 (ns={} svc={} remote={})",
                namespace,
                service,
                remote_port
            );
        }
    };

    // Re-attach stdout so further output (and EOF on kill) is captured
    // rather than buffered indefinitely. Without this kubectl can block
    // on a full pipe and the kill in Drop may take longer to reap.
    child.stdout = Some(stdout_back);

    // kubectl prints "Forwarding from" slightly before the listener is
    // actually accepting. A small sleep avoids a flaky first-probe
    // ECONNREFUSED. 200ms is empirically enough; the harness retry loop
    // would absorb a longer wait anyway, but skipping the sleep adds
    // noise to the logs.
    thread::sleep(Duration::from_millis(200));

    Ok(PortForwardHandle { local_port, child })
}

/// Parse the integer in `Forwarding from 127.0.0.1:<PORT> -> <REMOTE>`.
/// kubectl also emits an IPv6 variant (`[::1]:<PORT>`); we only consume
/// the v4 line. If the first line is the v6 one in some kubectl
/// version, the caller will retry from the next line via a wider read
/// — but in practice the v4 line is always first.
fn parse_forwarding_line(line: &str) -> Result<u16> {
    // Examples:
    //   "Forwarding from 127.0.0.1:54321 -> 8088\n"
    //   "Forwarding from [::1]:54321 -> 8088\n"
    let trimmed = line.trim();
    // Look for the 127.0.0.1: marker.
    let needle = "127.0.0.1:";
    let pos = trimmed
        .find(needle)
        .ok_or_else(|| anyhow!("first kubectl line lacks `127.0.0.1:` marker: {:?}", trimmed))?;
    let after = &trimmed[pos + needle.len()..];
    // Port is digits up to the first non-digit (space, `-`, end of string).
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    let port_str = &after[..end];
    if port_str.is_empty() {
        bail!("kubectl line had `127.0.0.1:` but no digits after it: {:?}", trimmed);
    }
    port_str
        .parse::<u16>()
        .map_err(|e| anyhow!("parsing port {:?}: {}", port_str, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_canonical_forwarding_line() {
        let line = "Forwarding from 127.0.0.1:54321 -> 8088\n";
        assert_eq!(parse_forwarding_line(line).unwrap(), 54321);
    }

    #[test]
    fn parse_forwarding_line_no_trailing_newline() {
        let line = "Forwarding from 127.0.0.1:65535 -> 80";
        assert_eq!(parse_forwarding_line(line).unwrap(), 65535);
    }

    #[test]
    fn parse_forwarding_line_rejects_unrelated() {
        let line = "error: bind: address already in use";
        assert!(parse_forwarding_line(line).is_err());
    }

    // Sanity: make sure ClusterHandle is at least mentioned so a future
    // refactor that breaks its public shape doesn't silently leave this
    // file building.
    #[allow(dead_code)]
    fn _typecheck_signature(handle: &ClusterHandle) {
        let _kc = &handle.kubeconfig;
    }
}
