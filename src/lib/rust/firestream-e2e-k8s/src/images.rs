//! Preload Nix-built container images into a per-test k3d cluster.
//!
//! The Firestream charts ship Nix-built `firestream-*` images that ONLY
//! exist as flake derivations on the developer's machine. They need to be:
//!
//! 1. Materialised into the host docker daemon's image store via
//!    `nix run .#<flake-attr>-image -- --load`.
//! 2. Side-loaded into the per-test k3d cluster via
//!    `k3d image import <repo>:<tag> -c <cluster>`.
//!
//! As of the Phase B chart migration, every chart's `chart-manifest.json`
//! `images` block references a `firestream-*` repository — there are no
//! more upstream Bitnami refs to handle. The `is_bitnami` skip-gate is
//! retained as defence in depth for any future chart that pulls
//! upstream-only images on demand.
//!
//! Respect `FIRESTREAM_E2E_K8S_PRELOAD=0` — skip the whole step. Useful
//! when running against a cluster that already has the images cached or
//! when iterating on the harness without rebuilding images each time.

use std::collections::HashSet;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use firestream_charts::ChartManifest;
use tracing::warn;

use crate::cluster::ClusterHandle;
use crate::env::env_preload;

/// One image that needs to be materialised + imported. Fields are:
///
/// - `flake_attr` — fully-qualified flake-app reference, e.g.
///   `.#airflow-image`. Derived from the image repository (NOT the slot
///   key — slot keys like `hub`/`proxy`/`singleuser` would map to
///   non-existent flake apps for multi-slot charts).
/// - `repo`       — image repository as it appears in `manifest.images`
///   (e.g. `firestream-airflow`). Used to form the `k3d image import` ref.
/// - `tag`        — image tag (e.g. `3.0.3`). Used to form the import ref.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PreloadTarget {
    flake_attr: String,
    repo: String,
    tag: String,
}

/// Pure helper extracted from `preload_images_for` for testability.
///
/// Walk the manifest's `images` map and derive the set of distinct
/// preload targets. We:
///
/// - Skip slots without a repository or tag (warn on the tag case).
/// - Skip Bitnami-shaped repositories (defence in depth; post-Phase-B
///   there should be none).
/// - Derive the flake attr via `flake_attr_stem(repo)` — this maps
///   `firestream-jupyterhub` → `.#jupyterhub-image`,
///   `firestream-os-shell` → `.#os-shell-image`, etc. The slot key
///   (`hub`, `proxy`, `singleuser`, `auxiliaryImage`, …) is NOT used —
///   multi-slot charts like jupyterhub have 3+ slots pointing at the
///   same flake app, and slot keys don't always match flake attrs.
/// - De-duplicate on `(flake_attr, repo, tag)` so the same image isn't
///   loaded multiple times when several slots reference it.
fn derive_preload_targets(manifest: &ChartManifest) -> HashSet<PreloadTarget> {
    let mut out: HashSet<PreloadTarget> = HashSet::new();
    for (slot, image) in &manifest.images {
        let repo = match image.repository.as_deref() {
            Some(r) if !r.is_empty() => r,
            _ => {
                // Slot without a repo override: nothing to materialise.
                continue;
            }
        };
        if is_bitnami(repo) {
            continue;
        }
        let tag = match image.tag.as_deref() {
            Some(t) if !t.is_empty() => t.to_string(),
            _ => {
                warn!(
                    chart = %manifest.name,
                    slot,
                    repo,
                    "image slot has repository but no tag; skipping preload"
                );
                continue;
            }
        };
        let flake_attr = format!(".#{}-image", flake_attr_stem(repo));
        out.insert(PreloadTarget {
            flake_attr,
            repo: repo.to_string(),
            tag,
        });
    }
    out
}

/// Preload every non-Bitnami image referenced by `manifest` into the cluster
/// behind `handle`. No-op when:
///
/// - `FIRESTREAM_E2E_K8S_PRELOAD=0` is set (operator opt-out).
/// - `manifest.images` is empty.
/// - Every image in the manifest is Bitnami-shaped (no current chart hits
///   this path post-Phase-B, but the guard remains).
///
/// For each distinct non-Bitnami image we:
///
/// 1. Run `nix run <flake-attr> -- --load` synchronously (stdio inherited
///    so the user sees progress; this can take minutes the first time and
///    is cached on subsequent runs). The flake attr is derived from the
///    image repository, not the slot key.
/// 2. Run `k3d image import <repository>:<tag> -c <handle.name>` to
///    side-load into the cluster's containerd.
///
/// Distinct images are de-duplicated upfront — jupyterhub references
/// `firestream-jupyterhub:5.3.0` from three slots (`hub`, `proxy`,
/// `singleuser`); we run `nix run .#jupyterhub-image` exactly once.
///
/// Returns `Err` with context on any failure.
pub fn preload_images_for(handle: &ClusterHandle, manifest: &ChartManifest) -> Result<()> {
    if !env_preload() {
        eprintln!(
            "[preload] FIRESTREAM_E2E_K8S_PRELOAD=0; skipping image preload for chart `{}`",
            manifest.name
        );
        return Ok(());
    }

    let targets = derive_preload_targets(manifest);
    if targets.is_empty() {
        eprintln!(
            "[preload] no non-bitnami images for chart `{}`; skipping",
            manifest.name
        );
        return Ok(());
    }

    for target in targets {
        let PreloadTarget { flake_attr, repo, tag } = target;
        eprintln!(
            "[preload] nix run {} -- --load (for {}:{})",
            flake_attr, repo, tag
        );
        let status = Command::new("nix")
            .args(["run", &flake_attr, "--", "--load"])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("spawn nix run {}", flake_attr))?;
        if !status.success() {
            bail!(
                "nix run {} -- --load failed with status {} (chart `{}` repo `{}`)",
                flake_attr,
                status,
                manifest.name,
                repo
            );
        }

        let image_ref = format!("{}:{}", repo, tag);
        eprintln!(
            "[preload] k3d image import {} -c {}",
            image_ref, handle.name
        );
        let status = Command::new("k3d")
            .args(["image", "import", &image_ref, "-c", &handle.name])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("spawn k3d image import {}", image_ref))?;
        if !status.success() {
            bail!(
                "k3d image import {} -c {} failed with status {}",
                image_ref,
                handle.name,
                status
            );
        }
    }

    Ok(())
}

/// Heuristic: treat any repo whose last `/`-segment is rooted at `bitnami/`
/// (or that contains `/bitnami/`, or starts with `bitnami/`) as a Bitnami
/// image that the cluster will pull from upstream on demand.
fn is_bitnami(repo: &str) -> bool {
    repo.starts_with("bitnami/")
        || repo.starts_with("docker.io/bitnami/")
        || repo.contains("/bitnami/")
}

/// Derive the Nix flake attribute base from an image repository name.
///
/// Two transformations:
///
/// 1. Strip any registry-namespace prefix (`myorg/firestream-airflow` →
///    `firestream-airflow`). Last `/`-segment wins.
/// 2. Strip the `firestream-` prefix (`firestream-airflow` → `airflow`,
///    `firestream-os-shell` → `os-shell`). The flake apps are registered
///    as `<short-name>-image` in `nix/flake-modules/charts/*.nix` and
///    `nix/flake-modules/containers/*.nix` — they do NOT carry the
///    `firestream-` prefix.
///
/// Multi-slot charts (jupyterhub: `hub`/`proxy`/`singleuser` all point at
/// `firestream-jupyterhub`, plus `auxiliaryImage` at `firestream-os-shell`)
/// rely on this — the slot key is unreliable as a flake-attr source.
fn flake_attr_stem(repo: &str) -> &str {
    let after_namespace = repo.rsplit('/').next().unwrap_or(repo);
    after_namespace.strip_prefix("firestream-").unwrap_or(after_namespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use firestream_charts::ImageSlot;
    use std::collections::BTreeMap;

    fn img(repository: &str, tag: &str) -> ImageSlot {
        ImageSlot {
            component_path: Vec::new(),
            registry: None,
            repository: Some(repository.to_string()),
            tag: Some(tag.to_string()),
        }
    }

    fn manifest_with(images: Vec<(&str, ImageSlot)>) -> ChartManifest {
        let mut map: BTreeMap<String, ImageSlot> = BTreeMap::new();
        for (slot, image) in images {
            map.insert(slot.to_string(), image);
        }
        ChartManifest {
            schema_version: "1".to_string(),
            name: "test-chart".to_string(),
            chart: "test-chart".to_string(),
            version: "0.0.0".to_string(),
            release: Default::default(),
            bundle: Default::default(),
            deployment: Default::default(),
            lifecycle: Default::default(),
            images: map,
            provenance: Default::default(),
        }
    }

    #[test]
    fn bitnami_root_repo_is_skipped() {
        assert!(is_bitnami("bitnami/postgresql"));
        assert!(is_bitnami("docker.io/bitnami/redis"));
        assert!(is_bitnami("registry.example.com/bitnami/kafka"));
    }

    #[test]
    fn firestream_repo_is_not_bitnami() {
        assert!(!is_bitnami("firestream-airflow"));
        assert!(!is_bitnami("myorg/firestream-airflow"));
    }

    #[test]
    fn flake_attr_stem_strips_namespace() {
        // Strip a registry namespace; if no `firestream-` prefix is left,
        // pass through.
        assert_eq!(flake_attr_stem("foo/some-repo"), "some-repo");
        assert_eq!(flake_attr_stem(""), "");
    }

    #[test]
    fn flake_attr_stem_strips_firestream_prefix() {
        assert_eq!(flake_attr_stem("firestream-airflow"), "airflow");
        assert_eq!(flake_attr_stem("firestream-os-shell"), "os-shell");
        assert_eq!(flake_attr_stem("firestream-jupyterhub"), "jupyterhub");
        assert_eq!(flake_attr_stem("firestream-postgresql"), "postgresql");
        // Strip namespace AND prefix:
        assert_eq!(flake_attr_stem("myorg/firestream-airflow"), "airflow");
    }

    #[test]
    fn derive_preload_targets_single_slot() {
        let m = manifest_with(vec![("airflow", img("firestream-airflow", "3.0.3"))]);
        let targets = derive_preload_targets(&m);
        assert_eq!(targets.len(), 1);
        let only = targets.iter().next().unwrap();
        assert_eq!(only.flake_attr, ".#airflow-image");
        assert_eq!(only.repo, "firestream-airflow");
        assert_eq!(only.tag, "3.0.3");
    }

    #[test]
    fn derive_preload_targets_jupyterhub_multi_slot_dedup() {
        // jupyterhub has hub/proxy/singleuser all pointing at the same
        // image, plus auxiliaryImage at os-shell, plus a postgres subchart
        // slot. Expect 3 distinct flake apps with no duplicate work.
        let m = manifest_with(vec![
            ("hub", img("firestream-jupyterhub", "5.3.0")),
            ("proxy", img("firestream-jupyterhub", "5.3.0")),
            ("singleuser", img("firestream-jupyterhub", "5.3.0")),
            ("auxiliaryImage", img("firestream-os-shell", "1")),
            ("postgresql", img("firestream-postgresql", "17")),
        ]);
        let targets = derive_preload_targets(&m);
        let attrs: HashSet<String> = targets.iter().map(|t| t.flake_attr.clone()).collect();
        assert_eq!(
            attrs,
            HashSet::from([
                ".#jupyterhub-image".to_string(),
                ".#os-shell-image".to_string(),
                ".#postgresql-image".to_string(),
            ]),
            "expected exactly 3 distinct flake apps; got: {:?}",
            attrs
        );
        assert_eq!(targets.len(), 3, "expected dedup to 3 targets; got {}", targets.len());
    }

    #[test]
    fn derive_preload_targets_skips_slot_without_repo() {
        let m = manifest_with(vec![
            ("good", img("firestream-airflow", "3.0.3")),
            (
                "no-repo",
                ImageSlot {
                    component_path: Vec::new(),
                    registry: None,
                    repository: None,
                    tag: Some("ignored".to_string()),
                },
            ),
        ]);
        let targets = derive_preload_targets(&m);
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn derive_preload_targets_skips_slot_without_tag() {
        let m = manifest_with(vec![
            ("good", img("firestream-airflow", "3.0.3")),
            (
                "no-tag",
                ImageSlot {
                    component_path: Vec::new(),
                    registry: None,
                    repository: Some("firestream-mystery".to_string()),
                    tag: None,
                },
            ),
        ]);
        let targets = derive_preload_targets(&m);
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn derive_preload_targets_skips_bitnami() {
        let m = manifest_with(vec![
            ("good", img("firestream-airflow", "3.0.3")),
            ("legacy", img("docker.io/bitnami/postgresql", "17.5.0")),
        ]);
        let targets = derive_preload_targets(&m);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets.iter().next().unwrap().repo, "firestream-airflow");
    }
}
