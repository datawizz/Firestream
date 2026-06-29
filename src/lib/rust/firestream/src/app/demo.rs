//! Demo/example-data env resolution for `firestream app`.
//!
//! Only three apps support demo data (their container env-defaults carry
//! a toggle): airflow (`AIRFLOW_LOAD_EXAMPLES`, default on), superset
//! (`SUPERSET_LOAD_EXAMPLES`, default on) and odoo (`ODOO_LOAD_DEMO_DATA`,
//! default off). The remaining five apps have no demo toggle.
//!
//! [`env_overrides`] turns an explicit `--demo`/`--no-demo` decision into
//! the `(VAR, "true"|"false")` env pair the docker/k8s backends inject.
//! Phase 3 consumes this; Phase 2 ships it complete (it's pure logic with
//! no orchestration) plus a stub for the not-yet-wired backends to call.

use super::registry::AppSpec;

/// Compute the env-var overrides for an explicit demo-data decision.
///
/// - `demo_on == None` → no override (use the container/chart default).
/// - `demo_on == Some(b)` on a demo-supporting app → `[(VAR, b)]`.
/// - `demo_on == Some(_)` on a non-demo app → empty (nothing to set;
///   callers should warn that the flag is a no-op for this app).
///
/// Returns owned `(String, String)` pairs so callers can move them
/// straight into a compose env map or a chart `extraEnvVars` list.
pub fn env_overrides(spec: &AppSpec, demo_on: Option<bool>) -> Vec<(String, String)> {
    match (spec.demo, demo_on) {
        (Some(var), Some(on)) => {
            vec![(var.to_string(), if on { "true" } else { "false" }.to_string())]
        }
        // No explicit decision, or app has no demo toggle: nothing to inject.
        _ => Vec::new(),
    }
}

/// Whether an explicit demo flag is a no-op for this app (i.e. the user
/// asked for `--demo`/`--no-demo` on an app that has no demo toggle).
/// Phase 3 uses this to emit a friendly warning.
pub fn is_demo_noop(spec: &AppSpec, demo_on: Option<bool>) -> bool {
    demo_on.is_some() && spec.demo.is_none()
}

#[cfg(test)]
mod tests {
    use super::super::registry::lookup;
    use super::*;

    #[test]
    fn demo_supported_app_sets_var() {
        let airflow = lookup("airflow").unwrap();
        assert_eq!(
            env_overrides(airflow, Some(true)),
            vec![("AIRFLOW_LOAD_EXAMPLES".to_string(), "true".to_string())]
        );
        assert_eq!(
            env_overrides(airflow, Some(false)),
            vec![("AIRFLOW_LOAD_EXAMPLES".to_string(), "false".to_string())]
        );
    }

    #[test]
    fn no_decision_is_empty() {
        let airflow = lookup("airflow").unwrap();
        assert!(env_overrides(airflow, None).is_empty());
    }

    #[test]
    fn non_demo_app_is_noop() {
        let postgres = lookup("postgresql").unwrap();
        assert!(env_overrides(postgres, Some(true)).is_empty());
        assert!(is_demo_noop(postgres, Some(true)));
        assert!(!is_demo_noop(postgres, None));
    }
}
