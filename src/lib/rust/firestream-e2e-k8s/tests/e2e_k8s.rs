//! Phase 3: cluster_lifecycle + postgresql + redis. Phase 4 fills in
//! the remaining 6 charts (airflow, kafka, spark, jupyterhub,
//! superset, odoo).
//!
//! Run individually:
//!
//! ```text
//! # Cluster lifecycle smoke (no chart deploy — cheaper than a full e2e):
//! cargo test -p firestream-e2e-k8s --test e2e_k8s -- \
//!     --ignored --nocapture e2e_k8s_cluster_lifecycle
//!
//! # Full chart e2e (deploy + wait + probe + teardown):
//! nix shell nixpkgs#k3d --command \
//!     cargo test -p firestream-e2e-k8s --test e2e_k8s -- \
//!         --ignored --test-threads=1 --nocapture e2e_k8s_postgresql
//! ```
//!
//! `k3d` is not on the devshell PATH yet (Phase-2 hand-off note); the
//! `nix shell nixpkgs#k3d --command …` wrap is a temporary workaround
//! until Phase 4 adds `k3d` to the devshell.

/// One `#[test] #[ignore]` per chart, all routed through
/// `harness::run_one`. Phase 4 will append more `stack_test_k8s!`
/// invocations as the probe chains for the remaining charts come
/// online.
macro_rules! stack_test_k8s {
    ($fn_name:ident, $chart:literal) => {
        #[test]
        #[ignore = "e2e-k8s: needs k3d+kubectl+helm+nix+docker; run via `nix shell nixpkgs#k3d --command cargo test -p firestream-e2e-k8s --test e2e_k8s -- --ignored --test-threads=1 --nocapture`"]
        fn $fn_name() {
            firestream_e2e_k8s::harness::run_one($chart);
        }
    };
}

/// Phase-2 carry-over: a no-deploy smoke that just creates and tears
/// down a cluster. Useful as a fast sanity check (the chart deploys
/// take 1-5 minutes each on a warm cache).
#[test]
#[ignore = "e2e-k8s: cluster smoke; cheaper than a full chart deploy"]
fn e2e_k8s_cluster_lifecycle() {
    // Bypass harness::run_one — we want to assert on the cluster
    // directly. The chart tests below go through run_one.
    use firestream_e2e_k8s::{cluster, env::env_keep_k8s, harness::should_skip_k8s};

    if let Some(reason) = should_skip_k8s() {
        eprintln!("[e2e-k8s:lifecycle] SKIP: {}", reason);
        return;
    }

    let handle = cluster::create_cluster("lifecycle").expect("create_cluster");
    eprintln!(
        "[e2e-k8s:lifecycle] cluster up: name={} kubeconfig={}",
        handle.name,
        handle.kubeconfig.display()
    );
    let _guard = cluster::ClusterGuard::arm(handle.clone(), env_keep_k8s());

    // Assert kubectl --kubeconfig <handle.kubeconfig> get nodes returns
    // at least one node. We don't bother asserting Ready here — k3d's
    // own setup_cluster path (which we don't take) is what blocks on
    // CoreDNS/etc.; for the lifecycle sentinel just proving the API
    // is reachable is the point.
    let output = std::process::Command::new("kubectl")
        .args(["--kubeconfig"])
        .arg(&handle.kubeconfig)
        .args(["get", "nodes", "-o", "name"])
        .output()
        .expect("kubectl get nodes");
    assert!(
        output.status.success(),
        "kubectl get nodes failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let nodes = String::from_utf8_lossy(&output.stdout);
    assert!(
        nodes.lines().count() >= 1,
        "expected >=1 node, got: {:?}",
        nodes
    );
    eprintln!("[e2e-k8s:lifecycle] nodes={}", nodes.lines().count());
}

// ---- Phase 3 charts ----
stack_test_k8s!(e2e_k8s_postgresql, "postgresql");
stack_test_k8s!(e2e_k8s_redis, "redis");

// ---- Phase 4 charts ----
stack_test_k8s!(e2e_k8s_kafka, "kafka");
stack_test_k8s!(e2e_k8s_airflow, "airflow");
stack_test_k8s!(e2e_k8s_spark, "spark");
stack_test_k8s!(e2e_k8s_jupyterhub, "jupyterhub");
stack_test_k8s!(e2e_k8s_superset, "superset");
stack_test_k8s!(e2e_k8s_odoo, "odoo");
