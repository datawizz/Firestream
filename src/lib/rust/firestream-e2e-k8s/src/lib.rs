//! End-to-end test harness for the Firestream chart bundle on k3d.
//!
//! Phase 2 of the k3d/helm e2e plan lands this crate with everything
//! needed to spin up and tear down a per-test k3d cluster. Subsequent
//! phases fill in chart deploy, image preload, port-forward, and the
//! per-chart probe matrix.
//!
//! # Layout
//!
//! - [`env`] — `FIRESTREAM_E2E_K8S_*` accessors. Distinct from core's
//!   `FIRESTREAM_E2E_*` because the docker and k8s harnesses can be
//!   filtered/configured independently.
//! - [`exec`] — `KubectlExec`: [`firestream_e2e_core::exec::Exec`] impl
//!   that shells out to `kubectl --kubeconfig <p> exec -n <ns> <pod>
//!   -- <args…>`. Lets the same `PgIsReady`/`RedisPing`/`KafkaApiVersions`
//!   probes from core drive both the docker and k8s harnesses.
//! - [`readiness`] — `wait_pods_ready`: blocking wrapper around
//!   `kubectl wait --for=condition=Ready pod -l app.kubernetes.io/instance=...`.
//! - [`cluster`] — per-test k3d cluster lifecycle. `create_cluster()`
//!   stands up a fresh cluster with a 6-char random suffix; `ClusterGuard`
//!   tears it down on Drop under a 60s budget (honors
//!   `FIRESTREAM_E2E_K8S_KEEP=1`).
//! - [`harness`] — `should_skip_k8s()` + `run_one(chart)` skeleton.
//!
//! Stubs for Phase 3+: [`deploy`], [`images`], [`portforward`], [`probes`].

pub mod cluster;
pub mod deploy;
pub mod env;
pub mod exec;
pub mod harness;
pub mod images;
pub mod portforward;
pub mod probes;
pub mod readiness;
