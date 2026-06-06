# Airflow chart options aggregator (Phase 4)
# Copyright Firestream. MIT License.
#
# Consumed as a NixOS module by `evalChart` (passed via `modules = [ ./nix/default.nix ]`).
# It imports all 25 option modules — each declaring `options.airflow.<section>.*`
# (Phase 3) — and projects the resolved option tree into the engine-declared
# `config.airflow.values` attrset that the values.yaml emitter serialises.
#
# CRITICAL — the `config` argument MUST be bound in the module signature below.
# The legacy `nix/modules/default.nix` was broken because it referenced `config`
# without binding it. We bind `config` here and read it back. Equally important:
# `builtins.removeAttrs config.airflow [ "_meta" "values" ]` strips those two keys
# BEFORE the recursive filter runs. `_meta`/`values` are declared by the engine's
# stdSchema, so leaving `values` in the tree would make `config.airflow.values`
# reference itself (infinite recursion). We read every OTHER section, drop nulls
# recursively, and write the result back to `values`.
{ lib, config, ... }:

{
  imports = [
    # Core Airflow configuration
    ./options/global.nix
    ./options/common.nix
    ./options/airflow-core.nix
    ./options/image.nix
    ./options/auth.nix
    ./options/dags.nix
    ./options/plugins.nix
    ./options/init-containers.nix
    ./options/sidecars.nix

    # Airflow components
    ./options/web.nix
    ./options/scheduler.nix
    ./options/dag-processor.nix
    ./options/triggerer.nix
    ./options/worker.nix
    ./options/setup-db-job.nix

    # Networking and exposure
    ./options/service.nix
    ./options/ingress.nix
    ./options/metrics.nix

    # Security and access
    ./options/rbac.nix
    ./options/ldap.nix
    ./options/service-account.nix

    # Subcharts and external services
    ./options/postgresql.nix
    ./options/redis.nix
    ./options/external-database.nix
    ./options/external-redis.nix
  ];

  # ------------------------------------------------------------------
  # Phase 1 — chart-manifest.json metadata (canary).
  #
  # These _meta knobs surface in `$out/chart-manifest.json` (built by the
  # engine via lib/to-chart-manifest.nix) and lock the v1 JSON contract
  # consumed by downstream agents (B: image injection; C: aggregate index;
  # D: Rust deploy layer).
  #
  # - deployment: helm install/upgrade flags. Airflow installs a DB-init job
  #   and benefits from `--wait --wait-for-jobs --atomic` so a failed init
  #   rolls back cleanly. 10m timeout accommodates DB init + image pulls.
  # - lifecycle.dependsOn: postgres + redis subcharts must converge first
  #   when they're externalised; harmless on the bundled deployment.
  # - lifecycle.lastBreakingVersion: no airflow handler exists in
  #   src/lib/rust/firestream/src/deploy/helm_lifecycle/charts/ (only
  #   prometheus/nginx/postgresql/kafka/external-dns), so we leave it null.
  # - containerRefs: Agent B (next phase) populates this from the container
  #   registry; kept as an empty attrset here so the manifest schema stays
  #   valid (the engine's null-strip would otherwise drop the field).
  # - provenance: framework.nix does not yet pass `inputs.self.rev` into
  #   evalChart, so both fields remain null. See the TODO in
  #   bin/nix/firestream/charts/eval-chart.nix.
  # ------------------------------------------------------------------
  config.airflow._meta.deployment = {
    atomic = true;
    wait = true;
    waitForJobs = true;
    timeout = "10m";
    forceUpgrade = false;
    hooksDisabled = false;
    skipCrds = false;
  };

  config.airflow._meta.lifecycle = {
    dependsOn = [ "postgresql" "redis" ];
    lastBreakingVersion = null;
  };

  # Project the resolved option tree (minus the engine's `_meta`/`values`) into
  # the serialised `values` attrset, stripping null leaves recursively. The
  # emitter (to-values-yaml.nix) does its own null-strip too, but filtering here
  # keeps `config.airflow.values` a faithful, null-free mirror of the overrides.
  config.airflow.values =
    lib.filterAttrsRecursive (_: v: v != null)
      (builtins.removeAttrs config.airflow [ "_meta" "values" ]);
}
