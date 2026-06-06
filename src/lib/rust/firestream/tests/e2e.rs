// End-to-end Docker compose tests for canonical Firestream stacks.
//
// Each #[test] below is gated by #[ignore] — they are opt-in because they
// drive `nix run .#<name>-up` / `-down`, which builds Linux container images
// in a Docker-from-Docker builder and brings up multi-service stacks. The
// canonical run is measured in HOURS on a cold cache; that is why e2e stays
// out of CI (see plan Phase 1).
//
// Run all:
//   make test-e2e                          # serialised; --test-threads=1
// Run one:
//   make test-e2e-postgresql               # or any canonical stack name
// Direct invocation:
//   cargo test -p firestream --test e2e -- --ignored --test-threads=1 --nocapture
//
// Each test calls into `harness::run_one(stack)` which handles:
//   1. Skip gate (nix + docker on PATH; docker daemon up)
//   2. Filter gate (FIRESTREAM_E2E_STACKS CSV)
//   3. Serialise on a global Mutex (correctness not dependent on -j1)
//   4. Discover ports/projectName/buildList via `nix eval --json`
//   5. `nix run .#<name>-down` (pre-clean), then `nix run .#<name>-up`
//   6. Arm a StackGuard for teardown
//   7. Run per-protocol readiness probes
//   8. Drop StackGuard → `nix run .#<name>-down` (with a 60s wall-clock
//      timeout and a `docker compose down -v` fallback).
//
// See `tests/e2e/harness.rs` for the precise sequence.
//
// Note: submodules are loaded via `#[path]` to keep the test binary's module
// tree explicit (avoids the `tests/e2e.rs` + `tests/e2e/mod.rs` ambiguity
// rustc rejects).

#[path = "e2e/stacks.rs"]
mod stacks;
#[path = "e2e/probes.rs"]
mod probes;
#[path = "e2e/harness.rs"]
mod harness;

macro_rules! stack_test {
    ($fn_name:ident, $stack:literal) => {
        #[test]
        #[ignore = "e2e: needs nix+docker; run via `make test-e2e` or `-- --ignored`"]
        fn $fn_name() {
            harness::run_one($stack);
        }
    };
}

stack_test!(e2e_airflow,    "airflow");
stack_test!(e2e_postgresql, "postgresql");
stack_test!(e2e_redis,      "redis");
stack_test!(e2e_kafka,      "kafka");
stack_test!(e2e_spark,      "spark");
stack_test!(e2e_jupyterhub, "jupyterhub");
stack_test!(e2e_superset,   "superset");
stack_test!(e2e_odoo,       "odoo");
