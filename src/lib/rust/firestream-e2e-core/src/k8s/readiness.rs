//! Pod-readiness wait helper.
//!
//! Bitnami helm charts surface `app.kubernetes.io/instance=<release>` on
//! every workload pod. The helm `--wait` flag only blocks until the
//! `Deployment` replica count is satisfied (and the firestream helm
//! lifecycle's `validate()` only checks pod `phase=Running`), neither
//! of which equals "the pod's containers report Ready". This module
//! adds the missing readiness check so Phase 3+ can synchronise probes
//! against actually-serving pods.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{Result, anyhow, bail};

/// Block until every pod with `app.kubernetes.io/instance=<release>` in
/// `namespace` reports `Ready`, or `deadline` elapses.
///
/// Shells out to:
///
/// ```text
/// kubectl --kubeconfig <kubeconfig> \
///   wait --for=condition=Ready pod \
///   -l app.kubernetes.io/instance=<release> \
///   -n <namespace> \
///   --timeout=<remaining-secs>s
/// ```
///
/// We pass the remaining deadline (in whole seconds, minimum 1) into
/// `kubectl wait --timeout` so the kubectl client and the harness agree
/// on when to give up. Returns `Err` with stderr captured on failure.
pub fn wait_pods_ready(
    kubeconfig: &Path,
    release: &str,
    namespace: &str,
    deadline: Instant,
) -> Result<()> {
    let now = Instant::now();
    if now >= deadline {
        bail!(
            "wait_pods_ready: deadline already elapsed for release={} ns={}",
            release,
            namespace
        );
    }
    // Whole seconds, minimum 1 — kubectl rejects `--timeout=0s` outright
    // and we don't want a sub-second budget either.
    let remaining_secs = deadline.saturating_duration_since(now).as_secs().max(1);
    let timeout_arg = format!("--timeout={}s", remaining_secs);
    let selector = format!("app.kubernetes.io/instance={}", release);

    let output = Command::new("kubectl")
        .arg("--kubeconfig")
        .arg(kubeconfig)
        .args([
            "wait",
            "--for=condition=Ready",
            "pod",
            "-l",
            &selector,
            "-n",
            namespace,
            &timeout_arg,
        ])
        .output()
        .map_err(|e| anyhow!("spawn kubectl wait: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "kubectl wait failed (release={} ns={} status={}): stderr: {} stdout: {}",
            release,
            namespace,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            String::from_utf8_lossy(&output.stdout).trim()
        ))
    }
}
