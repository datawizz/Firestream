//! Deploy a single chart from the bundle onto a per-test k3d cluster.
//!
//! Phase 3 of the k3d/helm e2e plan. Mirrors the CLI helm-deploy call
//! sequence at `src/lib/rust/firestream/src/cli/commands.rs:1066-1134`:
//!
//! ```text
//! Charts::open(charts_dir)?
//!     .get(chart)?                       -> ChartManifest
//! chart_info_from_manifest(&manifest)    -> ChartInfo
//! CommonChart::new(info)                 -> Box<dyn HelmChart>
//! deploy_chart_lifecycle(common, Some(kubeconfig)).await
//! ```
//!
//! Routing the test through `deploy_chart_lifecycle` (rather than
//! shelling out to `helm install` directly) is a constraint of the plan:
//! the point is to validate the exact code path the CLI takes.
//!
//! `deploy_chart_lifecycle` is async, so we wrap it in a current-thread
//! tokio runtime. The public API of this crate stays synchronous —
//! matching `firestream-e2e-core`'s convention and avoiding
//! tokio-dropped-during-unwind hazards in the harness.

use std::path::Path;

use anyhow::{Context, Result, bail};
use firestream::deploy::helm::deploy_chart_lifecycle;
use firestream::deploy::helm_lifecycle::{CommonChart, chart_info_from_manifest};
use firestream_charts::Charts;
use tracing::info;

use crate::cluster::ClusterHandle;
use crate::images;

/// What a successful deploy hands back to the harness. The fields are
/// exactly what `wait_pods_ready` + the probe chain need:
///
/// - `chart`     — Firestream chart name (debug logging only).
/// - `release`   — Helm release name from the manifest, falling back to
///   the chart name. Drives the
///   `app.kubernetes.io/instance=<release>` label selector.
/// - `namespace` — Manifest namespace, falling back to `default`. Drives
///   `kubectl -n` for waits/probes and the namespace-delete on teardown.
#[derive(Debug, Clone)]
pub struct DeployedRelease {
    pub chart: String,
    pub release: String,
    pub namespace: String,
}

/// Deploy `chart` from the bundle at `charts_dir` onto the cluster behind
/// `handle`. Returns the resolved release info so the harness can pass
/// the correct release/namespace to `wait_pods_ready` and the probe
/// chain.
///
/// Steps:
///
/// 1. Open the bundle. `Charts::open` parses `index.json`; per-chart
///    manifests are loaded lazily.
/// 2. Look up the manifest for `chart`.
/// 3. Preload non-Bitnami images into the cluster's containerd (Phase 3
///    no-op for postgres/redis; Phase 4's airflow exercises this).
/// 4. Convert the manifest to a `ChartInfo` (identical conversion the
///    CLI uses).
/// 5. Wrap in `CommonChart` and feed to `deploy_chart_lifecycle` under a
///    current-thread tokio runtime, passing the per-test kubeconfig so
///    the deploy targets THIS cluster (not whatever
///    `$KUBECONFIG`/`~/.kube/config` says).
/// 6. Build the `DeployedRelease` from the manifest's release block
///    (fallbacks documented above).
pub fn deploy_chart(
    handle: &ClusterHandle,
    charts_dir: &Path,
    chart: &str,
) -> Result<DeployedRelease> {
    let charts = Charts::open(charts_dir).with_context(|| {
        format!(
            "open chart bundle at {} (set FIRESTREAM_E2E_K8S_CHARTS_DIR or run \
             `nix build .#firestream-charts-bundle --out-link <dir>`)",
            charts_dir.display()
        )
    })?;

    let manifest = charts
        .get(chart)
        .with_context(|| format!("look up chart `{}` in bundle {}", chart, charts_dir.display()))?;

    // Materialise the firestream-* images BEFORE the helm install kicks
    // off — otherwise the cluster will try to pull them from upstream and
    // fail (they only exist as local flake derivations). Post-Phase-B
    // every chart has a populated `images` block, so this drives a real
    // load for every deploy.
    images::preload_images_for(handle, &manifest)
        .with_context(|| format!("preload images for chart `{}`", chart))?;

    let mut info = chart_info_from_manifest(&manifest);
    // The chart manifest's `deployment.timeout` is authoritative. The
    // `FIRESTREAM_E2E_K8S_HELM_TIMEOUT` env var is an emergency-override
    // path only — leave it unset under normal use.
    if let Some(t) = crate::env::env_helm_timeout() {
        info.timeout = t;
    }
    let release_name = info.name.clone();
    let namespace = info.custom_namespace.clone().unwrap_or_else(|| {
        // chart_info_from_manifest leaves custom_namespace=None when the
        // manifest doesn't set release.namespace. Fall back to the helm
        // default rather than the bundle's chart name — matches what
        // helm install would do absent any `-n` flag.
        "default".to_string()
    });

    info!(
        chart = %chart,
        release = %release_name,
        namespace = %namespace,
        chart_path = ?manifest.bundle.chart_path,
        "deploying chart via helm lifecycle"
    );

    // deploy_chart_lifecycle is async; wrap in a current-thread runtime
    // so this function stays sync per firestream-e2e-core's convention.
    // The runtime is dropped at the end of this block, well before any
    // unwind path the caller might hit.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio current-thread runtime for helm deploy")?;

    let kubeconfig = handle.kubeconfig.clone();
    let common = CommonChart::new(info);

    rt.block_on(async move { deploy_chart_lifecycle(Box::new(common), Some(&kubeconfig)).await })
        .map_err(|e| {
            // `FirestreamError` doesn't implement `std::error::Error +
            // Send + Sync + 'static` cleanly across versions; stringify
            // for a uniform anyhow surface.
            anyhow::anyhow!("deploy_chart_lifecycle failed for `{}`: {}", chart, e)
        })?;

    // Sanity: the manifest must agree on the release name we'll feed to
    // the readiness wait + probe selectors. If chart_info_from_manifest
    // ever stops using `manifest.name` as the release, this guard fires
    // loudly rather than letting the probe chain silently look at the
    // wrong pods.
    if release_name.is_empty() {
        bail!(
            "manifest for chart `{}` produced empty release name; \
             cannot drive readiness selector",
            chart
        );
    }

    Ok(DeployedRelease {
        chart: chart.to_string(),
        release: release_name,
        namespace,
    })
}
