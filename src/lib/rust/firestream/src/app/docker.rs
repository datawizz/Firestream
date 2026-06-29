//! Docker / docker-compose backend for `firestream app`.
//!
//! Each handler drives the Nix-emitted compose stack for the app
//! (`.#<spec.compose_attr>-compose` / `.#<spec.compose_attr>-up`), reusing
//! the exact discovery approach the docker e2e harness uses
//! (`tests/e2e/harness.rs`'s `nix_eval_json` / `nix_build_compose`).
//!
//! # Lifecycle contract
//!
//! - `up` brings the stack up and LEAVES it running (no teardown). With
//!   `--wait` it then runs `health`.
//! - `down` tears the stack down (`nix run .#<attr>-down`).
//! - `health` probes the published `healthHostPort` (`/readyz`) when the
//!   container's healthd is enabled, else falls back to a TCP probe of the
//!   first published host port.
//!
//! # Demo-data override mechanism
//!
//! Three apps (airflow / superset / odoo) ship a demo-data env toggle.
//! When the caller forces `--demo`/`--no-demo` and that differs from the
//! container-baked default, we generate a tiny `docker-compose.override.yml`
//! in a temp dir that sets the demo env var on every service of the project,
//! then run `docker compose -f <composeFile> -f <override> -p <project> up -d`.
//!
//! Setting the var on every service (via a YAML anchor expanded per service
//! is not portable, so we enumerate services) is overkill but robust: the
//! var is only meaningful to the app's own container and is a harmless no-op
//! elsewhere. We discover the service list from `docker compose config
//! --services` against the rendered compose file. The container env-defaults
//! emit `export VAR="${VAR:-default}"`, so a compose-injected value wins.

use std::path::PathBuf;
use std::process::Stdio;

use super::demo;
use super::registry::AppSpec;
use super::AppOpts;
use crate::core::{FirestreamError, Result};

use firestream_e2e_core::probe::{HttpReady, Probe, Tcp};
use firestream_e2e_core::retry::retry_until_sync;
use std::time::{Duration, Instant};

/// Resolved per-app compose info read from Nix (mirrors the e2e harness's
/// `StackInfo`, minus the build list which `nix run .#<attr>-up` handles).
struct ComposeInfo {
    compose_file: PathBuf,
    project: String,
    host_ports: Vec<u16>,
    health_host_port: Option<u16>,
}

/// `nix eval --json <attr>` → parsed JSON. Copied from the e2e harness.
fn nix_eval_json(attr: &str) -> Result<serde_json::Value> {
    let out = std::process::Command::new("nix")
        .args(["eval", "--json", attr])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| FirestreamError::GeneralError(format!("spawn nix eval {}: {}", attr, e)))?;
    if !out.status.success() {
        return Err(FirestreamError::GeneralError(format!(
            "nix eval {} exited {}",
            attr, out.status
        )));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| FirestreamError::GeneralError(format!("parse nix eval {}: {}", attr, e)))
}

/// `nix build --no-link --print-out-paths .#<attr>-compose` → the rendered
/// `docker-compose.yml`. Copied from the e2e harness.
fn nix_build_compose(attr: &str) -> Result<PathBuf> {
    let out = std::process::Command::new("nix")
        .args([
            "build",
            "--no-link",
            "--print-out-paths",
            &format!(".#{}-compose", attr),
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| {
            FirestreamError::GeneralError(format!("spawn nix build .#{}-compose: {}", attr, e))
        })?;
    if !out.status.success() {
        return Err(FirestreamError::GeneralError(format!(
            "nix build .#{}-compose exited {}",
            attr, out.status
        )));
    }
    let store_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if store_path.is_empty() {
        return Err(FirestreamError::GeneralError(format!(
            "nix build .#{}-compose produced no output path",
            attr
        )));
    }
    Ok(PathBuf::from(store_path).join("docker-compose.yml"))
}

/// Resolve the full compose info for an app from Nix.
fn resolve_compose(attr: &str) -> Result<ComposeInfo> {
    let ports_v = nix_eval_json(&format!(".#{}-compose.passthru.hostPorts", attr))?;
    let host_ports: Vec<u16> = ports_v
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_u64())
                .filter_map(|n| u16::try_from(n).ok())
                .collect()
        })
        .unwrap_or_default();

    let project_v = nix_eval_json(&format!(".#{}-compose.passthru.projectName", attr))?;
    let project = project_v
        .as_str()
        .ok_or_else(|| {
            FirestreamError::GeneralError(format!("projectName for {} is not a string", attr))
        })?
        .to_string();

    // `healthHostPort` is null when the container's healthd is disabled.
    let health_host_port = match nix_eval_json(&format!(
        ".#{}-compose.passthru.healthHostPort",
        attr
    )) {
        Ok(v) if v.is_null() => None,
        Ok(v) => v.as_u64().and_then(|n| u16::try_from(n).ok()),
        Err(_) => None,
    };

    let compose_file = nix_build_compose(attr)?;

    Ok(ComposeInfo {
        compose_file,
        project,
        host_ports,
        health_host_port,
    })
}

/// Whether an explicit `--demo`/`--no-demo` decision differs from the
/// app's container-baked default and therefore needs a compose override.
///
/// Baked defaults (per container env-defaults): airflow on, superset on,
/// odoo off. Returns `false` when the app has no demo toggle or the caller
/// didn't force a decision.
fn demo_override_needed(spec: &AppSpec, opts: &AppOpts) -> bool {
    let forced = match opts.demo {
        Some(v) => v,
        None => return false,
    };
    let baked_default = match spec.name {
        "airflow" => Some(true),
        "superset" => Some(true),
        "odoo" => Some(false),
        _ => None,
    };
    match baked_default {
        Some(d) => d != forced,
        // App has no demo toggle: nothing to override.
        None => false,
    }
}

/// Enumerate the compose project's service names so the override file can
/// set the demo env var on each one.
fn compose_services(compose_file: &std::path::Path, project: &str) -> Result<Vec<String>> {
    let out = std::process::Command::new("docker")
        .args(["compose", "-f"])
        .arg(compose_file)
        .args(["-p", project, "config", "--services"])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| {
            FirestreamError::GeneralError(format!("spawn docker compose config --services: {}", e))
        })?;
    if !out.status.success() {
        return Err(FirestreamError::GeneralError(format!(
            "docker compose config --services failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Write a `docker-compose.override.yml` setting `var=value` on every
/// service. Returns the temp dir (kept alive by the caller) and the
/// override file path.
fn write_demo_override(
    services: &[String],
    var: &str,
    value: &str,
) -> Result<(tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir().map_err(|e| {
        FirestreamError::GeneralError(format!("create temp dir for compose override: {}", e))
    })?;
    let path = dir.path().join("docker-compose.override.yml");
    let mut yaml = String::from("services:\n");
    for svc in services {
        yaml.push_str(&format!("  {}:\n", svc));
        yaml.push_str("    environment:\n");
        yaml.push_str(&format!("      {}: \"{}\"\n", var, value));
    }
    std::fs::write(&path, yaml).map_err(|e| {
        FirestreamError::GeneralError(format!("write compose override: {}", e))
    })?;
    Ok((dir, path))
}

/// `nix run .#<attr>-up` (builds + loads images, then `docker compose up -d`).
fn nix_run_up(attr: &str) -> Result<()> {
    let status = std::process::Command::new("nix")
        .args(["run", &format!(".#{}-up", attr)])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| FirestreamError::GeneralError(format!("spawn nix run .#{}-up: {}", attr, e)))?;
    if status.success() {
        Ok(())
    } else {
        Err(FirestreamError::GeneralError(format!(
            "nix run .#{}-up exited {}",
            attr, status
        )))
    }
}

/// Bring the app up via docker-compose, leaving it running.
pub async fn up(spec: &AppSpec, opts: &AppOpts) -> Result<()> {
    // Warn on a no-op demo flag (app has no toggle).
    if demo::is_demo_noop(spec, opts.demo) {
        eprintln!(
            "[app:{}] note: --demo/--no-demo is a no-op for this app (no demo-data toggle); ignoring",
            spec.name
        );
    }

    let attr = spec.compose_attr;
    let needs_override = demo_override_needed(spec, opts);

    // Always run `nix run .#<attr>-up` first to ensure images are built and
    // loaded. When no override is needed this also brings the stack up.
    println!("[app:{}] up via docker: nix run .#{}-up", spec.name, attr);
    nix_run_up(attr)?;

    if needs_override {
        // Re-run compose with the demo env override layered on. Discover
        // compose file + project + service list, generate the override,
        // and `up -d` again (idempotent — recreates only changed services).
        let info = resolve_compose(attr)?;
        let pairs = demo::env_overrides(spec, opts.demo);
        if let Some((var, value)) = pairs.into_iter().next() {
            println!(
                "[app:{}] applying demo override {}={} via docker-compose.override.yml",
                spec.name, var, value
            );
            let services = compose_services(&info.compose_file, &info.project)?;
            // `tmp` (the TempDir) must outlive the docker invocation; it is
            // dropped at the end of this block, cleaning up the override file.
            let (tmp, override_path) = write_demo_override(&services, &var, &value)?;
            let status = std::process::Command::new("docker")
                .args(["compose", "-f"])
                .arg(&info.compose_file)
                .arg("-f")
                .arg(&override_path)
                .args(["-p", &info.project, "up", "-d"])
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| {
                    FirestreamError::GeneralError(format!("spawn docker compose up -d: {}", e))
                })?;
            drop(tmp);
            if !status.success() {
                return Err(FirestreamError::GeneralError(format!(
                    "docker compose up -d (with demo override) exited {}",
                    status
                )));
            }
        }
    }

    // Print where the app is reachable.
    let info = resolve_compose(attr)?;
    if let Some(hp) = info.health_host_port {
        println!("[app:{}] healthd: http://localhost:{}/readyz", spec.name, hp);
    }
    if let Some(p) = info.host_ports.first() {
        println!("[app:{}] URL: http://localhost:{}", spec.name, p);
    }
    if !info.host_ports.is_empty() {
        println!("[app:{}] published host ports: {:?}", spec.name, info.host_ports);
    }

    if opts.wait {
        health(spec, opts).await?;
    }
    Ok(())
}

/// Tear the app's compose stack down. Plain `down` (volumes preserved so
/// demo/example data survives a restart). Use docker directly to drop
/// volumes if that's ever needed.
pub async fn down(spec: &AppSpec, _opts: &AppOpts) -> Result<()> {
    let attr = spec.compose_attr;
    println!("[app:{}] down via docker: nix run .#{}-down", spec.name, attr);
    let status = std::process::Command::new("nix")
        .args(["run", &format!(".#{}-down", attr)])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            FirestreamError::GeneralError(format!("spawn nix run .#{}-down: {}", attr, e))
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(FirestreamError::GeneralError(format!(
            "nix run .#{}-down exited {}",
            attr, status
        )))
    }
}

/// Probe the app's health over docker. Prints PASS/FAIL and errors on FAIL.
///
/// The underlying probes are synchronous and `HttpReady` internally spins
/// up (and drops) a `reqwest::blocking` runtime, which panics if dropped on
/// a tokio worker thread. We therefore run the whole probe loop inside
/// `spawn_blocking` so it never touches the async context.
pub async fn health(spec: &AppSpec, opts: &AppOpts) -> Result<()> {
    let info = resolve_compose(spec.compose_attr)?;
    let timeout_secs = opts.timeout_secs.max(1);
    let name = spec.name.to_string();
    let health_host_port = info.health_host_port;
    let first_port = info.host_ports.first().copied();

    let result = tokio::task::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        if let Some(hp) = health_host_port {
            let url = format!("http://127.0.0.1:{}/readyz", hp);
            println!("[app:{}] health: probing {} (healthd)", name, url);
            let probe = HttpReady {
                url,
                max_status: 500,
            };
            Ok(retry_until_sync(&deadline, || probe.run()))
        } else if let Some(port) = first_port {
            println!(
                "[app:{}] health: healthd disabled, TCP-probing first host port {}",
                name, port
            );
            let probe = Tcp { port };
            Ok(retry_until_sync(&deadline, || probe.run()))
        } else {
            Err(())
        }
    })
    .await
    .map_err(|e| FirestreamError::GeneralError(format!("probe task join error: {}", e)))?;

    let result = match result {
        Ok(r) => r,
        Err(()) => {
            return Err(FirestreamError::GeneralError(format!(
                "[app:{}] health: no healthHostPort and no published host ports to probe",
                spec.name
            )));
        }
    };

    match result {
        Ok(()) => {
            println!("[app:{}] health: PASS", spec.name);
            Ok(())
        }
        Err(e) => {
            println!("[app:{}] health: FAIL ({})", spec.name, e);
            Err(FirestreamError::GeneralError(format!(
                "[app:{}] docker health probe failed: {}",
                spec.name, e
            )))
        }
    }
}

/// Best-effort per-app status line: `docker compose -p <project> ps`.
pub fn status_line(spec: &AppSpec) {
    let attr = spec.compose_attr;
    let project = match nix_eval_json(&format!(".#{}-compose.passthru.projectName", attr)) {
        Ok(v) => v.as_str().map(|s| s.to_string()),
        Err(_) => None,
    };
    let project = match project {
        Some(p) => p,
        None => {
            println!("[app:{}] docker: <unknown project (nix eval failed)>", spec.name);
            return;
        }
    };
    let out = std::process::Command::new("docker")
        .args(["compose", "-p", &project, "ps", "--format", "{{.Name}} {{.State}}"])
        .stdin(Stdio::null())
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let body = String::from_utf8_lossy(&o.stdout);
            let trimmed = body.trim();
            if trimmed.is_empty() {
                println!("[app:{}] docker (project {}): not running", spec.name, project);
            } else {
                println!("[app:{}] docker (project {}):", spec.name, project);
                for line in trimmed.lines() {
                    println!("    {}", line);
                }
            }
        }
        _ => println!("[app:{}] docker (project {}): <ps failed>", spec.name, project),
    }
}
