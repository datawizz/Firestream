# helm-manager

This crate is the **helm-CLI execution layer** for Firestream. It provides
thin async Rust wrappers around the `helm` and `kubectl` binaries plus the
data types and builders that describe what to deploy.

For **chart discovery** (i.e. which charts exist, what their values shape is,
where their files live on disk) see the `firestream-charts` crate, which
reads the flake-emitted chart index at `/opt/firestream/charts`.

## What lives here

- `helm_client::HelmClient` — async wrapper around `helm install / upgrade /
  uninstall / rollback / status / list`.
- `kubectl_client::KubectlClient` — async wrapper around `kubectl`.
- `values_resolver::resolve_values` — merges values from files, env vars
  (with a `.env` file), and inline overrides into a single JSON payload.
- `Deployment` / `Stack` builders and the supporting `Release`,
  `ReleaseStatus`, `Values`, `Chart`, `ChartMetadata` data types.
- `traits` — `HelmClientTrait`, `KubectlClientTrait`, `ValuesProvider` for
  swap-in mocks/alternatives.

## What used to live here (and where it went)

Previous versions of this crate embedded a fork of the Bitnami chart archive
at compile time via `include_dir!` and exposed an `EmbeddedCharts` /
`ChartManager` / `HelmManager` orchestration layer. That subsystem is gone:
the Nix flake is now the single source of truth for charts (and for
containers and compose specs), and the Rust side reads the flake's outputs
via `firestream-charts`. The `build.rs`, `embedded_charts` module,
`chart_manager` module, and the `HelmManager` aggregator struct have all
been removed.

## Requirements

- Rust 2024 edition
- `helm` and `kubectl` installed and available in PATH
- Access to a Kubernetes cluster (for actual deployments)

## Usage sketch

```rust,no_run
use helm_manager::helm_client::HelmClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), helm_manager::Error> {
    let helm = HelmClient::new()?;
    let release = helm
        .install(
            "my-database",
            "/opt/firestream/charts/postgresql",
            "default",
            json!({}),
            true,  // wait
            false, // atomic
        )
        .await?;
    println!("Deployed: {}", release.name);
    Ok(())
}
```

## License

MIT
