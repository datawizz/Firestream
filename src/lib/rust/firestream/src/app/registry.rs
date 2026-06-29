//! Canonical Firestream app registry.
//!
//! The eight apps below are the single source of truth shared by the
//! `firestream app` CLI subcommands (and, from Phase 3 onward, by the
//! docker/k8s orchestration). Each [`AppSpec`] ties together:
//!
//! - the **Nix compose attr base** (`compose_attr`): the docker backend
//!   drives `.#<compose_attr>-compose` / `.#<compose_attr>-up`.
//! - the **chart name** (`chart`): the k8s backend resolves this against
//!   the firestream-charts bundle `index.json`.
//! - the **demo env var** (`demo`): `Some(<VAR>)` for the three apps that
//!   support demo/example data, `None` otherwise. Only airflow, superset
//!   and odoo ship a demo-data toggle in their container env-defaults.
//! - the **k8s namespace** the chart deploys into by default.

/// Metadata for one canonical Firestream app.
#[derive(Debug, Clone, Copy)]
pub struct AppSpec {
    /// Stable app name as accepted on the CLI (e.g. `airflow`).
    pub name: &'static str,
    /// Chart name as registered in the firestream-charts `index.json`.
    pub chart: &'static str,
    /// Nix attr base for the docker backend. The backend appends
    /// `-compose` / `-up` (e.g. `airflow` -> `.#airflow-up`).
    pub compose_attr: &'static str,
    /// Demo/example-data env var name, or `None` if the app has no
    /// demo-data toggle. `Some("AIRFLOW_LOAD_EXAMPLES")` etc.
    pub demo: Option<&'static str>,
    /// Default Kubernetes namespace for the k8s backend.
    pub k8s_namespace: &'static str,
}

impl AppSpec {
    /// Whether this app supports demo/example data.
    pub fn supports_demo(&self) -> bool {
        self.demo.is_some()
    }
}

/// The eight canonical Firestream apps.
///
/// Demo support (per container env-defaults): only airflow
/// (`AIRFLOW_LOAD_EXAMPLES`, default on), superset
/// (`SUPERSET_LOAD_EXAMPLES`, default on) and odoo
/// (`ODOO_LOAD_DEMO_DATA`, default off) carry a demo toggle.
pub const ALL: &[AppSpec] = &[
    AppSpec {
        name: "airflow",
        chart: "airflow",
        compose_attr: "airflow",
        demo: Some("AIRFLOW_LOAD_EXAMPLES"),
        k8s_namespace: "default",
    },
    AppSpec {
        name: "postgresql",
        chart: "postgresql",
        compose_attr: "postgresql",
        demo: None,
        k8s_namespace: "default",
    },
    AppSpec {
        name: "redis",
        chart: "redis",
        compose_attr: "redis",
        demo: None,
        k8s_namespace: "default",
    },
    AppSpec {
        name: "kafka",
        chart: "kafka",
        compose_attr: "kafka",
        demo: None,
        k8s_namespace: "default",
    },
    AppSpec {
        name: "spark",
        chart: "spark",
        compose_attr: "spark",
        demo: None,
        k8s_namespace: "default",
    },
    AppSpec {
        name: "jupyterhub",
        chart: "jupyterhub",
        compose_attr: "jupyterhub",
        demo: None,
        k8s_namespace: "default",
    },
    AppSpec {
        name: "superset",
        chart: "superset",
        compose_attr: "superset",
        demo: Some("SUPERSET_LOAD_EXAMPLES"),
        k8s_namespace: "default",
    },
    AppSpec {
        name: "odoo",
        chart: "odoo",
        compose_attr: "odoo",
        demo: Some("ODOO_LOAD_DEMO_DATA"),
        k8s_namespace: "default",
    },
];

/// Look up a single app by exact name.
pub fn lookup(name: &str) -> Option<&'static AppSpec> {
    ALL.iter().find(|s| s.name == name)
}

/// Resolve a CLI selector to a list of app specs.
///
/// The literal `all` expands to every canonical app; any other value is
/// treated as a single app name (returns an empty `Vec` if unknown — the
/// caller is responsible for reporting "unknown app").
pub fn resolve(selector: &str) -> Vec<&'static AppSpec> {
    if selector == "all" {
        ALL.iter().collect()
    } else {
        lookup(selector).into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_eight_canonical_apps() {
        assert_eq!(ALL.len(), 8);
    }

    #[test]
    fn only_three_apps_support_demo() {
        let with_demo: Vec<&str> = ALL
            .iter()
            .filter(|s| s.supports_demo())
            .map(|s| s.name)
            .collect();
        assert_eq!(with_demo, vec!["airflow", "superset", "odoo"]);
    }

    #[test]
    fn lookup_known_and_unknown() {
        assert_eq!(lookup("kafka").map(|s| s.chart), Some("kafka"));
        assert!(lookup("nope").is_none());
    }

    #[test]
    fn resolve_all_expands() {
        assert_eq!(resolve("all").len(), 8);
        assert_eq!(resolve("odoo").len(), 1);
        assert!(resolve("nope").is_empty());
    }
}
