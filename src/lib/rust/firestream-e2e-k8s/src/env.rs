//! Env-var contract accessors specific to the k8s harness.
//!
//! These mirror the shape of `firestream_e2e_core::env::*` but read
//! `FIRESTREAM_E2E_K8S_*` keys so the k8s harness can be filtered /
//! configured independently of the docker-compose harness running in
//! the same workspace.
//!
//! All accessors read process env at call-time. Tests that mutate env
//! must serialise (see the `ENV_LOCK` mutex below) because
//! `std::env::set_var` is process-global.
//!
//! | Var                                | Type   | Default | Meaning                                                          |
//! |------------------------------------|--------|---------|------------------------------------------------------------------|
//! | `FIRESTREAM_E2E_K8S_STRICT`        | bool   | unset   | `1` ⇒ skip-gate failure becomes a hard panic                     |
//! | `FIRESTREAM_E2E_K8S_KEEP`          | bool   | unset   | `1` ⇒ harness skips cluster + release teardown                   |
//! | `FIRESTREAM_E2E_K8S_TIMEOUT_SECS`  | u64    | 600     | Per-chart readiness deadline (probe + wait phase)                |
//! | `FIRESTREAM_E2E_K8S_PRELOAD`       | bool   | true    | `0` ⇒ skip Nix image preload (Phase 3+ uses this)                |
//! | `FIRESTREAM_E2E_K8S_CHARTS_DIR`    | path   | varies  | Bundle override; falls back to `FIRESTREAM_CHARTS_DIR`, then `/opt/firestream/charts` |
//! | `FIRESTREAM_E2E_K8S_STACKS`        | CSV    | unset   | Filter gate (matched in [`selected_k8s`])                        |

use std::path::PathBuf;

/// `FIRESTREAM_E2E_K8S_STRICT=1` upgrades a skip-gate miss (missing tool
/// on PATH, dead docker daemon, …) into a hard panic. Unset is "skip
/// politely" so a developer machine without k3d can still `cargo test`
/// the workspace.
pub fn env_strict_k8s() -> bool {
    std::env::var("FIRESTREAM_E2E_K8S_STRICT").ok().as_deref() == Some("1")
}

/// `FIRESTREAM_E2E_K8S_KEEP=1` skips teardown of both the per-test
/// cluster AND any deployed releases (Phase 3+). Use when an operator
/// wants to inspect a failing cluster after the test exits.
pub fn env_keep_k8s() -> bool {
    std::env::var("FIRESTREAM_E2E_K8S_KEEP").ok().as_deref() == Some("1")
}

/// `FIRESTREAM_E2E_K8S_TIMEOUT_SECS=<u64>` overrides the per-chart
/// readiness deadline. Default 600s. Bounds the wait+probe phase only;
/// cluster create / helm install have their own budgets inside their
/// respective libraries.
pub fn env_timeout_secs_k8s() -> u64 {
    std::env::var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(600)
}

/// `FIRESTREAM_E2E_K8S_HELM_TIMEOUT=<go-duration>` overrides the helm
/// install timeout that the deploy path picks up from the chart manifest.
///
/// **Emergency override only.** The chart manifest's `deployment.timeout`
/// (typically `"5m"`) is authoritative. Set this env var only when a
/// specific chart needs more headroom than its manifest declares — and
/// prefer fixing the manifest. Returns `None` when unset/empty so the
/// deploy path falls back to the manifest default.
pub fn env_helm_timeout() -> Option<String> {
    std::env::var("FIRESTREAM_E2E_K8S_HELM_TIMEOUT")
        .ok()
        .filter(|s| !s.is_empty())
}

/// `FIRESTREAM_E2E_K8S_PRELOAD=0` disables the Nix-built image preload
/// step (Phase 3+). Default is "preload on" — set explicitly to `"0"`
/// to opt out (any other value, including `"1"`, `""`, or unset, leaves
/// preload enabled).
pub fn env_preload() -> bool {
    match std::env::var("FIRESTREAM_E2E_K8S_PRELOAD").ok() {
        Some(v) if v == "0" => false,
        _ => true,
    }
}

/// Resolve the chart bundle directory. Resolution order:
///
/// 1. `FIRESTREAM_E2E_K8S_CHARTS_DIR` (k8s-harness-specific override)
/// 2. `FIRESTREAM_CHARTS_DIR` (project-wide; set by the Nix devshell)
/// 3. `/opt/firestream/charts` (production deploy path)
///
/// Returning a `PathBuf` (rather than `Option<PathBuf>`) keeps the
/// call-site ergonomics simple — Phase 3's deploy code can either
/// `.exists()` check it or hand it straight to `Charts::open`, which
/// will surface a clear "no index.json" error if the path is bogus.
pub fn env_charts_dir() -> PathBuf {
    if let Ok(p) = std::env::var("FIRESTREAM_E2E_K8S_CHARTS_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(p) = std::env::var("FIRESTREAM_CHARTS_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("/opt/firestream/charts")
}

/// `FIRESTREAM_E2E_K8S_STACKS=all|csv` filter gate.
///
/// Behaviour (mirrors core's [`firestream_e2e_core::env::selected`]):
/// - var unset / empty / `"all"` → selected (true)
/// - var is CSV → match `stack` against any trimmed token (case-sensitive)
///
/// Unlike core's variant we don't take an `is_canonical` predicate
/// because Phase 2 has only the cluster-lifecycle test — there is no
/// canonical chart set to consult yet. Phase 3 may wrap this with a
/// canonical-chart filter; today "unset" means "let it through" so the
/// cluster-lifecycle test always runs when no filter is provided.
pub fn selected_k8s(stack: &str) -> bool {
    match std::env::var("FIRESTREAM_E2E_K8S_STACKS").ok() {
        None => true,
        Some(v) if v.trim().is_empty() => true,
        Some(v) if v.trim() == "all" => true,
        Some(v) => v.split(',').any(|s| s.trim() == stack),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialise env mutation so cargo's in-process parallel runner
    // can't trample these tests against each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock<'a>() -> std::sync::MutexGuard<'a, ()> {
        ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner())
    }

    // ---- strict ----

    #[test]
    fn strict_default_when_unset() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_STRICT");
        }
        assert!(!env_strict_k8s());
    }

    #[test]
    fn strict_honored_when_set() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_STRICT", "1");
        }
        let got = env_strict_k8s();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_STRICT");
        }
        assert!(got);
    }

    // ---- keep ----

    #[test]
    fn keep_default_when_unset() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_KEEP");
        }
        assert!(!env_keep_k8s());
    }

    #[test]
    fn keep_honored_when_set() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_KEEP", "1");
        }
        let got = env_keep_k8s();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_KEEP");
        }
        assert!(got);
    }

    // ---- timeout ----

    #[test]
    fn timeout_default_when_unset() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS");
        }
        assert_eq!(env_timeout_secs_k8s(), 600);
    }

    #[test]
    fn timeout_honored_when_set() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS", "120");
        }
        let got = env_timeout_secs_k8s();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS");
        }
        assert_eq!(got, 120);
    }

    #[test]
    fn timeout_falls_back_on_nonsense() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS", "not-a-number");
        }
        let got = env_timeout_secs_k8s();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_TIMEOUT_SECS");
        }
        assert_eq!(got, 600);
    }

    // ---- preload ----

    #[test]
    fn preload_default_is_true_when_unset() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_PRELOAD");
        }
        assert!(env_preload());
    }

    #[test]
    fn preload_only_zero_disables() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_PRELOAD", "0");
        }
        let got_zero = env_preload();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_PRELOAD", "1");
        }
        let got_one = env_preload();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_PRELOAD");
        }
        assert!(!got_zero, "PRELOAD=0 must disable");
        assert!(got_one, "PRELOAD=1 must remain enabled");
    }

    // ---- charts_dir ----

    #[test]
    fn charts_dir_default_to_opt() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_CHARTS_DIR");
            std::env::remove_var("FIRESTREAM_CHARTS_DIR");
        }
        assert_eq!(env_charts_dir(), PathBuf::from("/opt/firestream/charts"));
    }

    #[test]
    fn charts_dir_honors_k8s_override_first() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_CHARTS_DIR", "/tmp/k8s-charts");
            std::env::set_var("FIRESTREAM_CHARTS_DIR", "/tmp/generic-charts");
        }
        let got = env_charts_dir();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_CHARTS_DIR");
            std::env::remove_var("FIRESTREAM_CHARTS_DIR");
        }
        assert_eq!(got, PathBuf::from("/tmp/k8s-charts"));
    }

    #[test]
    fn charts_dir_falls_back_to_generic_when_k8s_unset() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_CHARTS_DIR");
            std::env::set_var("FIRESTREAM_CHARTS_DIR", "/tmp/generic-charts");
        }
        let got = env_charts_dir();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_CHARTS_DIR");
        }
        assert_eq!(got, PathBuf::from("/tmp/generic-charts"));
    }

    // ---- selected ----

    #[test]
    fn selected_unset_means_true() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_STACKS");
        }
        assert!(selected_k8s("postgresql"));
        assert!(selected_k8s("lifecycle"));
    }

    #[test]
    fn selected_all_means_true() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_STACKS", "all");
        }
        let got = selected_k8s("anything");
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_STACKS");
        }
        assert!(got);
    }

    #[test]
    fn selected_csv_honored() {
        let _g = lock();
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_K8S_STACKS", "postgresql, redis");
        }
        let in_list = selected_k8s("redis");
        let not_in_list = selected_k8s("kafka");
        // SAFETY: env access serialised via ENV_LOCK.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_K8S_STACKS");
        }
        assert!(in_list);
        assert!(!not_in_list);
    }
}
