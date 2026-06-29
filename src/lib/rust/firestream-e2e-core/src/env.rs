//! Env-var contract accessors shared by every harness.
//!
//! All accessors read process env at call-time. Tests that want to assert
//! against env mutation should serialise their access through a shared
//! `Mutex` because `std::env::set_var` is process-global.
//!
//! Variables interpreted here (subset of `FIRESTREAM_E2E_*` consumed by core):
//!
//! | Var                            | Type   | Default | Meaning                                                    |
//! |--------------------------------|--------|---------|------------------------------------------------------------|
//! | `FIRESTREAM_E2E_STRICT`        | bool   | unset   | `1` ⇒ skip-gate failure becomes a hard panic               |
//! | `FIRESTREAM_E2E_KEEP`          | bool   | unset   | `1` ⇒ harness skips teardown                               |
//! | `FIRESTREAM_E2E_TIMEOUT_SECS`  | u64    | 300     | Per-stack readiness deadline                               |
//! | `FIRESTREAM_E2E_STACKS`        | CSV    | unset   | Filter gate (consumed by [`selected`])                     |
//!
//! Backend-specific overlays (e.g. `FIRESTREAM_E2E_K8S_STACKS`) live in their
//! own crate's env module — this crate only owns the names common to every
//! harness.

use std::process::Command;
use std::process::Stdio;

/// `FIRESTREAM_E2E_STRICT=1` upgrades skip-gate failures to a hard panic.
pub fn env_strict() -> bool {
    std::env::var("FIRESTREAM_E2E_STRICT").ok().as_deref() == Some("1")
}

/// `FIRESTREAM_E2E_KEEP=1` skips teardown so an operator can inspect a
/// running stack after the test exits.
pub fn env_keep() -> bool {
    std::env::var("FIRESTREAM_E2E_KEEP").ok().as_deref() == Some("1")
}

/// `FIRESTREAM_E2E_TIMEOUT_SECS=<u64>` overrides the per-stack readiness
/// deadline. Default: 300s. Note this bounds the probe phase only — it does
/// NOT bound nix/docker/helm build time.
pub fn env_timeout_secs() -> u64 {
    std::env::var("FIRESTREAM_E2E_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300)
}

/// `FIRESTREAM_E2E_STACKS=all|csv` filter gate, transport-agnostic.
///
/// Behaviour:
/// - var unset / empty / `"all"` → return `is_canonical(stack)`
/// - var is CSV → match `stack` against any trimmed token (case-sensitive)
///
/// `is_canonical` is a caller-supplied predicate so this function doesn't
/// have to know which stack set it's filtering against (the docker harness
/// has a different canonical set from any future per-backend harness).
pub fn selected<F>(stack: &str, is_canonical: F) -> bool
where
    F: Fn(&str) -> bool,
{
    match std::env::var("FIRESTREAM_E2E_STACKS").ok() {
        None => is_canonical(stack),
        Some(v) if v.trim().is_empty() => is_canonical(stack),
        Some(v) if v.trim() == "all" => is_canonical(stack),
        Some(v) => v.split(',').any(|s| s.trim() == stack),
    }
}

/// Generic skip-gate: returns `Some(reason)` if any binary in
/// `required_binaries` is missing from `PATH`, else `None`.
///
/// Backend-specific liveness checks (e.g. `docker info` for the docker
/// harness, `kubectl cluster-info` for kubectl) MUST be added by the
/// caller on top of this — that's why this function only inspects
/// `PATH` and never spawns the binaries it found.
pub fn should_skip(required_binaries: &[&str]) -> Option<String> {
    for bin in required_binaries {
        if which::which(bin).is_err() {
            return Some(format!("{} not on PATH", bin));
        }
    }
    None
}

/// Convenience wrapper around `docker info` for backends that need the
/// daemon to be reachable. Kept here (rather than in a docker-specific
/// module) because the docker harness is the only current consumer and
/// the implementation is trivial — promote out if a third caller appears.
///
/// Returns `Some(reason)` if `docker` is missing OR the daemon is
/// unreachable; `None` if the daemon answered.
pub fn docker_daemon_check() -> Option<String> {
    if which::which("docker").is_err() {
        return Some("docker not on PATH".into());
    }
    let info = Command::new("docker")
        .args(["info"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match info {
        Ok(s) if s.success() => None,
        Ok(s) => Some(format!("docker info exited with {}", s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some("docker binary not found".into()),
        Err(e) => Some(format!("docker info spawn failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // `std::env::set_var` is process-global; serialise the env-mutating
    // tests so they don't trample each other when cargo runs them in
    // parallel within the same process.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn timeout_default_when_unset() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // Use a unique-per-test key under the documented name so this
        // test reflects the real accessor; remove on exit.
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_TIMEOUT_SECS");
        }
        assert_eq!(env_timeout_secs(), 300);
    }

    #[test]
    fn timeout_honored_when_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_TIMEOUT_SECS", "42");
        }
        let got = env_timeout_secs();
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_TIMEOUT_SECS");
        }
        assert_eq!(got, 42);
    }

    #[test]
    fn timeout_falls_back_on_nonsense() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_TIMEOUT_SECS", "not-a-number");
        }
        let got = env_timeout_secs();
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_TIMEOUT_SECS");
        }
        assert_eq!(got, 300);
    }

    #[test]
    fn selected_falls_back_to_canonical_when_unset() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_STACKS");
        }
        let canonical = |s: &str| s == "postgresql" || s == "redis";
        assert!(selected("postgresql", canonical));
        assert!(!selected("airflow", canonical));
    }

    #[test]
    fn selected_csv_honored() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::set_var("FIRESTREAM_E2E_STACKS", "airflow, redis");
        }
        let canonical = |_: &str| true;
        let got_redis = selected("redis", canonical);
        let got_kafka = selected("kafka", canonical);
        // SAFETY: env access is serialised via ENV_LOCK in this module.
        unsafe {
            std::env::remove_var("FIRESTREAM_E2E_STACKS");
        }
        assert!(got_redis);
        assert!(!got_kafka);
    }

    #[test]
    fn should_skip_reports_missing_binary() {
        // Pick a binary name we're certain is not on PATH anywhere.
        let reason = should_skip(&["definitely-not-a-real-bin-xyzzy-fizzbuzz"]);
        assert!(reason.is_some());
    }
}
