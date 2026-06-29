//! `firestream app` — deploy / test / demo a canonical Firestream app
//! against either the docker or the kubernetes backend.
//!
//! # Layout
//!
//! - [`registry`] — the eight-app source of truth ([`registry::AppSpec`],
//!   [`registry::ALL`], [`registry::lookup`], [`registry::resolve`]).
//! - [`docker`] — docker/compose backend (Phase 3).
//! - [`k8s`] — kubernetes (k3d) backend (Phase 3).
//! - [`demo`] — demo/example-data env-var resolution (Phase 3).
//!
//! Phase 2 wires the clap subcommands through [`execute_app_command`] and
//! fully implements `list`. `up`/`down`/`health`/`test`/`status` call into
//! the backend modules, whose implementations are honest stubs printing a
//! "not yet implemented (Phase 3)" line.

pub mod registry;
pub mod docker;
pub mod k8s;
pub mod demo;

use crate::cli::args::{AppCommand, Backend, Cli};
use crate::core::{FirestreamError, Result};

use registry::AppSpec;

/// Resolved per-invocation options threaded into the backend handlers.
///
/// `demo` is `None` when neither `--demo` nor `--no-demo` was passed
/// (meaning "use the app's default"), `Some(true)`/`Some(false)` when one
/// was. The mutually-exclusive `--demo` / `--no-demo` pair is resolved
/// here (in [`resolve_demo`]) rather than in clap so the precedence is
/// explicit and testable.
#[derive(Debug, Clone)]
pub struct AppOpts {
    /// Demo-data override: `None` = app default, `Some(on)` = forced.
    pub demo: Option<bool>,
    /// Block until the app reports healthy.
    pub wait: bool,
    /// Operation/readiness timeout in seconds.
    pub timeout_secs: u64,
    /// (k8s only) Use an ephemeral per-run k3d cluster.
    pub ephemeral: bool,
    /// Namespace override (k8s backend); falls back to the spec default.
    pub namespace: Option<String>,
    /// (k8s only) Chart bundle directory, threaded from `cli.charts_dir`.
    /// `None` falls back to `$FIRESTREAM_CHARTS_DIR` / the default path.
    pub charts_dir: Option<std::path::PathBuf>,
    /// (k8s only) Preload the chart's `firestream-*` images into the target
    /// cluster's containerd before the helm deploy. `false` when the user
    /// passed `--no-preload` or set `FIRESTREAM_APP_PRELOAD=0`. Defaults to
    /// `true` for `up`/`test`; `false` for ops that don't deploy
    /// (`down`/`health`/`status`).
    pub preload: bool,
}

/// Resolve the effective preload setting: the `--no-preload` flag disables
/// it, and `FIRESTREAM_APP_PRELOAD=0` disables it independently. Either one
/// turns preload off.
fn resolve_preload(no_preload: bool) -> bool {
    if no_preload {
        return false;
    }
    !matches!(std::env::var("FIRESTREAM_APP_PRELOAD").ok().as_deref(), Some("0"))
}

/// Resolve the `--demo` / `--no-demo` flag pair into a tri-state.
///
/// `--demo` and `--no-demo` are mutually exclusive; passing both is a
/// user error. Passing neither yields `None` (use the app default).
fn resolve_demo(demo: bool, no_demo: bool) -> Result<Option<bool>> {
    match (demo, no_demo) {
        (true, true) => Err(FirestreamError::GeneralError(
            "--demo and --no-demo are mutually exclusive".to_string(),
        )),
        (true, false) => Ok(Some(true)),
        (false, true) => Ok(Some(false)),
        (false, false) => Ok(None),
    }
}

/// Human label for a backend, used in stub/status output.
fn backend_label(b: Backend) -> &'static str {
    match b {
        Backend::Docker => "docker",
        Backend::K8s => "k8s",
    }
}

/// Resolve a CLI selector into concrete app specs, erroring on an
/// unknown app name. `all` expands to every canonical app.
fn resolve_apps(selector: &str) -> Result<Vec<&'static AppSpec>> {
    let specs = registry::resolve(selector);
    if specs.is_empty() {
        let known: Vec<&str> = registry::ALL.iter().map(|s| s.name).collect();
        return Err(FirestreamError::ConfigError(format!(
            "unknown app `{}`; known apps: {} (or `all`)",
            selector,
            known.join(", ")
        )));
    }
    Ok(specs)
}

/// Dispatch a `firestream app <subcommand>`.
pub async fn execute_app_command(command: &AppCommand, cli: &Cli) -> Result<()> {
    match command {
        AppCommand::List => {
            print_list();
            Ok(())
        }

        AppCommand::Status { backend } => {
            status(*backend, cli).await
        }

        AppCommand::Up {
            app,
            backend,
            demo,
            no_demo,
            wait,
            timeout,
            ephemeral,
            namespace,
            no_preload,
        } => {
            let opts = AppOpts {
                demo: resolve_demo(*demo, *no_demo)?,
                wait: *wait,
                timeout_secs: *timeout,
                ephemeral: *ephemeral,
                namespace: namespace.clone().or_else(|| cli.namespace.clone()),
                charts_dir: Some(cli.charts_dir.clone()),
                preload: resolve_preload(*no_preload),
            };
            for spec in resolve_apps(app)? {
                dispatch_up(spec, *backend, &opts).await?;
            }
            Ok(())
        }

        AppCommand::Down {
            app,
            backend,
            ephemeral,
            namespace,
        } => {
            let opts = AppOpts {
                demo: None,
                wait: false,
                timeout_secs: 0,
                ephemeral: *ephemeral,
                namespace: namespace.clone().or_else(|| cli.namespace.clone()),
                charts_dir: Some(cli.charts_dir.clone()),
                preload: false,
            };
            for spec in resolve_apps(app)? {
                dispatch_down(spec, *backend, &opts).await?;
            }
            Ok(())
        }

        AppCommand::Health {
            app,
            backend,
            timeout,
            namespace,
        } => {
            let opts = AppOpts {
                demo: None,
                wait: true,
                timeout_secs: *timeout,
                ephemeral: false,
                namespace: namespace.clone().or_else(|| cli.namespace.clone()),
                charts_dir: Some(cli.charts_dir.clone()),
                preload: false,
            };
            for spec in resolve_apps(app)? {
                dispatch_health(spec, *backend, &opts).await?;
            }
            Ok(())
        }

        AppCommand::Test {
            app,
            backend,
            demo,
            no_demo,
            wait,
            timeout,
            ephemeral,
            namespace,
            no_preload,
        } => {
            let opts = AppOpts {
                demo: resolve_demo(*demo, *no_demo)?,
                wait: *wait,
                timeout_secs: *timeout,
                ephemeral: *ephemeral,
                namespace: namespace.clone().or_else(|| cli.namespace.clone()),
                charts_dir: Some(cli.charts_dir.clone()),
                preload: resolve_preload(*no_preload),
            };
            // `all` aggregates: run every app, collect failures, and only
            // error at the end so one bad app doesn't mask the rest.
            let specs = resolve_apps(app)?;
            let mut failures: Vec<String> = Vec::new();
            for spec in specs {
                if let Err(e) = dispatch_test(spec, *backend, &opts).await {
                    eprintln!("[app:{}] test FAILED: {}", spec.name, e);
                    failures.push(spec.name.to_string());
                }
            }
            if failures.is_empty() {
                Ok(())
            } else {
                Err(FirestreamError::GeneralError(format!(
                    "app test failed for: {}",
                    failures.join(", ")
                )))
            }
        }
    }
}

/// Print the canonical app table: `app | backends | demo`.
fn print_list() {
    println!("{:<12}  {:<14}  {:<13}  {}", "APP", "BACKENDS", "DEMO-DATA", "CHART");
    for spec in registry::ALL {
        let demo = match spec.demo {
            Some(var) => format!("yes ({})", var),
            None => "no".to_string(),
        };
        println!(
            "{:<12}  {:<14}  {:<13}  {}",
            spec.name, "docker, k8s", demo, spec.chart
        );
    }
}

// ---------- backend dispatch (stubs delegate to docker/k8s modules) ----------

async fn dispatch_up(spec: &AppSpec, backend: Backend, opts: &AppOpts) -> Result<()> {
    match backend {
        Backend::Docker => docker::up(spec, opts).await,
        Backend::K8s => k8s::up(spec, opts).await,
    }
}

async fn dispatch_down(spec: &AppSpec, backend: Backend, opts: &AppOpts) -> Result<()> {
    match backend {
        Backend::Docker => docker::down(spec, opts).await,
        Backend::K8s => k8s::down(spec, opts).await,
    }
}

async fn dispatch_health(spec: &AppSpec, backend: Backend, opts: &AppOpts) -> Result<()> {
    match backend {
        Backend::Docker => docker::health(spec, opts).await,
        Backend::K8s => k8s::health(spec, opts).await,
    }
}

/// `test` = up + health. LEAVES the deployment running (no teardown). When
/// `--wait` is set (the default for test), `up` itself runs health; when it
/// isn't, we still drive health explicitly so `test` always reports
/// readiness and returns non-zero on failure (so `make app-test-*` fails
/// loudly).
async fn dispatch_test(spec: &AppSpec, backend: Backend, opts: &AppOpts) -> Result<()> {
    println!("[app:{}] test via {} (up + health)", spec.name, backend_label(backend));
    dispatch_up(spec, backend, opts).await?;
    // If `up` already ran health (wait), don't double-probe; otherwise probe now.
    if !opts.wait {
        dispatch_health(spec, backend, opts).await?;
    }
    Ok(())
}

/// `status` prints a per-app one-line status per backend. Best-effort: a
/// backend that can't be reached prints a `<…failed>` marker rather than
/// erroring the whole command.
async fn status(backend: Option<Backend>, cli: &Cli) -> Result<()> {
    let opts = AppOpts {
        demo: None,
        wait: false,
        timeout_secs: 0,
        ephemeral: false,
        namespace: cli.namespace.clone(),
        charts_dir: Some(cli.charts_dir.clone()),
        preload: false,
    };

    let show_docker = matches!(backend, None | Some(Backend::Docker));
    let show_k8s = matches!(backend, None | Some(Backend::K8s));

    if show_docker {
        println!("=== docker ===");
        for spec in registry::ALL {
            docker::status_line(spec);
        }
    }
    if show_k8s {
        println!("=== k8s ===");
        for spec in registry::ALL {
            k8s::status_line(spec, &opts);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_demo_tri_state() {
        assert_eq!(resolve_demo(false, false).unwrap(), None);
        assert_eq!(resolve_demo(true, false).unwrap(), Some(true));
        assert_eq!(resolve_demo(false, true).unwrap(), Some(false));
        assert!(resolve_demo(true, true).is_err());
    }

    #[test]
    fn resolve_apps_all_and_single() {
        assert_eq!(resolve_apps("all").unwrap().len(), 8);
        assert_eq!(resolve_apps("kafka").unwrap().len(), 1);
        assert!(resolve_apps("bogus").is_err());
    }
}
