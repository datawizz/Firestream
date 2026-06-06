# PostgreSQL chart flake-module (Phase 6b - Agent G1)
# Copyright Firestream. MIT License.
#
# Wires the postgresql Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.postgresql-chart          (deployable chart bundle; builds on darwin)
#   - firestreamCharts.postgresql        (full evaluated chart result, for aggregate)
#   - firestreamChartImages.postgresql   (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/airflow.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# Image injection is NOT yet wired (no firestreamImages.postgresql container in
# the registry). Per Agent B's hand-off, postgresql/redis follow the single-image
# pattern with `componentPath = [ "image" ]` — that overlay can be added
# trivially once the firestream-postgresql container exists. For now,
# `_meta.containerRefs` stays empty and Bitnami's bundled `bitnami/postgresql`
# image renders normally; no `global.security.allowInsecureImages` flip needed.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/postgresql;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. PostgreSQL's Chart.yaml
      # lists `common` as its only dependency; the engine's vendor-subcharts.nix
      # handles the nested copy.
      subcharts = [
        { name = "common"; }
      ];

      c = evalChart {
        name = "postgresql";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.postgresql-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.postgresql = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.postgresql.
      firestreamChartImages.postgresql = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "postgresql";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
