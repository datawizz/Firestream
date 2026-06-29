//! Kubernetes / k3d glue shared by the k8s e2e harness (and, from Phase 1
//! onward, available to the main `firestream` CLI crate).
//!
//! These modules were relocated from `firestream-e2e-k8s` so the main
//! `firestream` crate can reuse the cluster lifecycle + probe matrix
//! WITHOUT depending on `firestream-e2e-k8s` (which itself depends on
//! `firestream` via `deploy.rs`, so a reverse edge would form a cycle).
//! Everything here imports only `firestream_e2e_core`'s own primitives
//! plus std / external crates and shells out to `kubectl` / `k3d` —
//! nothing here knows about the `firestream` crate.
//!
//! # Modules
//!
//! - [`cluster`] — per-test k3d cluster lifecycle: [`cluster::create_cluster`],
//!   [`cluster::ClusterHandle`], [`cluster::ClusterGuard`].
//! - [`exec`] — [`exec::KubectlExec`]: a [`crate::exec::Exec`] impl that
//!   shells `kubectl --kubeconfig <p> exec`.
//! - [`images`] — preload Nix-built `firestream-*` images into a target
//!   cluster's containerd ([`images::preload_images`],
//!   [`images::ImportTarget`]); k3d (`k3d image import`) and host-k3s
//!   (`docker save` → `ctr … images import`) backends.
//! - [`portforward`] — RAII `kubectl port-forward` wrapper.
//! - [`readiness`] — `kubectl wait --for=condition=Ready` helper.
//! - [`probes`] — the per-chart probe matrix ([`probes::for_chart`]) and
//!   the [`probes::K8sCtx`] context type.

pub mod cluster;
pub mod exec;
pub mod images;
pub mod portforward;
pub mod probes;
pub mod readiness;

// Flatten the most-used types so callers can `use
// firestream_e2e_core::k8s::{...}` without chasing submodule paths.
pub use cluster::{ClusterGuard, ClusterHandle, create_cluster};
pub use exec::KubectlExec;
pub use images::{ImportTarget, preload_images, preload_images_for};
pub use portforward::{PortForwardHandle, forward_service};
pub use probes::{K8sCtx, Probe, for_chart};
pub use readiness::wait_pods_ready;
