//! `kubectl exec`-backed [`Exec`] impl for the k8s harness.
//!
//! Mirrors the `DockerComposeExec` reference impl in the docker harness
//! (`src/lib/rust/firestream/tests/e2e/probes.rs`). Both satisfy
//! [`firestream_e2e_core::exec::Exec`] so the same protocol-level
//! probes (`PgIsReady`, `RedisPing`, `KafkaApiVersions`) drive both
//! backends with only the `target` interpretation differing —
//! compose-service-name vs pod-name.
//!
//! The kubeconfig path is held on the struct (rather than read from
//! the environment) so per-test clusters can each have their own
//! kubeconfig without trampling `$KUBECONFIG`.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result};
use crate::exec::Exec;

/// `Exec` impl that runs `kubectl --kubeconfig <kubeconfig> exec -n
/// <namespace> <target> -- <args…>`.
///
/// `target` is interpreted as a pod name (not a deployment/service —
/// callers that want to pick a pod from a label selector should resolve
/// to a concrete name first via `kubectl get pods -l ... -o name`).
pub struct KubectlExec {
    /// Namespace passed via `-n`. Held on the struct so a per-release
    /// instance can be stashed in an `Arc<dyn Exec>` and shared with
    /// every probe in that release's chain.
    pub namespace: String,
    /// Path to a kubeconfig file. We always pass `--kubeconfig` rather
    /// than relying on `$KUBECONFIG` so concurrent per-test clusters
    /// can't collide on a single shared default.
    pub kubeconfig: PathBuf,
}

impl Exec for KubectlExec {
    fn exec(&self, target: &str, args: &[&str]) -> Result<Output> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("--kubeconfig")
            .arg(&self.kubeconfig)
            .args(["exec", "-n", &self.namespace, target, "--"])
            .args(args)
            .stdin(Stdio::null());
        cmd.output()
            .with_context(|| format!("spawn kubectl exec {} {:?}", target, args))
    }
}
