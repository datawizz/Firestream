//! Transport-agnostic command execution against a service in a remote runtime.
//!
//! The shape `exec(target, &[args])` is deliberately minimal: it covers both
//! `docker compose -f <file> -p <project> exec -T <service> <args…>` and
//! `kubectl exec -n <ns> <pod> -- <args…>`. The trait must stay object-safe
//! (`&dyn Exec`) so callers can hold heterogeneous backends behind a single
//! pointer — keep methods non-generic and avoid `Self: Sized` bounds.
//!
//! `target` is opaque to this trait. In the docker backend it's the compose
//! service name; in the kubectl backend it's a pod name. The caller picks the
//! interpretation when constructing the `Exec` impl.

use std::process::Output;

use anyhow::Result;

/// Run a command inside a service/container/pod managed by some backend.
///
/// `target` identifies the destination (compose service, pod, …).
/// `args` is the command + arguments to run inside the target.
/// Implementations return the raw `Output` (status + stdout + stderr) and let
/// callers decide whether a non-zero exit is fatal.
pub trait Exec: Send + Sync {
    fn exec(&self, target: &str, args: &[&str]) -> Result<Output>;
}
