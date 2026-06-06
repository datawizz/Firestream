# Redis chart flake-module (Phase 6b - Agent G2)
# Copyright Firestream. MIT License.
#
# Wires the redis Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.redis-chart                (deployable chart bundle; builds on darwin)
#   - firestreamCharts.redis              (full evaluated chart result, for aggregate)
#   - firestreamChartImages.redis         (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/postgresql.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# Image injection is NOT yet wired (no firestreamImages.redis container in
# the registry). Per Agent B's hand-off, postgresql/redis follow the single-image
# pattern with `componentPath = [ "image" ]` — that overlay can be added
# trivially once the firestream-redis container exists. For now,
# `_meta.containerRefs` stays empty and Bitnami's bundled
# `bitnami/redis` + `bitnami/redis-sentinel` + `bitnami/redis-exporter` images
# render normally; no `global.security.allowInsecureImages` flip needed.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/redis;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. Redis's Chart.yaml
      # lists `common` as its only dependency; the engine's vendor-subcharts.nix
      # handles the nested copy.
      subcharts = [
        { name = "common"; }
      ];

      c = evalChart {
        name = "redis";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.redis-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.redis = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.redis.
      firestreamChartImages.redis = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "redis";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
