# Superset chart flake-module (Phase 6b - Agent G6)
# Copyright Firestream. MIT License.
#
# Wires the Superset Helm chart through the options-driven evalChart
# entrypoint and contributes:
#   - packages.superset-chart            (deployable chart bundle; builds on darwin)
#   - firestreamCharts.superset          (full evaluated chart result, for aggregate)
#   - firestreamChartImages.superset     (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/jupyterhub.nix. No isLinux gate —
# `helm template` runs in the build sandbox on every platform.
#
# The Bitnami Superset chart bundles `common` + `postgresql` + `redis` as
# vendored subcharts (Chart.yaml v4.0.1 lists all three). vendor-subcharts.nix
# consumes the in-repo Bitnami fork at src/charts/bitnami/. The recursive
# vendoring handles postgresql's own `common` dependency.
#
# Image injection is NOT yet wired (no firestreamImages.superset container
# in the registry has its image consumed here). The Bitnami chart ships ONE
# main image (bitnami/superset) shared across web/worker/beat/flower/init;
# that would be injected (componentPath = [ "image" ]) once a Firestream
# container exists. For now, `_meta.containerRefs` stays empty and the
# bundled Bitnami image renders unchanged; no
# `global.security.allowInsecureImages` flip needed.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/superset;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. Superset's
      # Chart.yaml lists `redis` (21.x.x), `postgresql` (16.x.x), and
      # `common` (2.x.x). The engine's vendor-subcharts.nix handles the
      # nested copy AND postgresql's own `common` dependency.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
        { name = "redis"; }
      ];

      c = evalChart {
        name = "superset";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.superset-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.superset = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.superset.
      firestreamChartImages.superset = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "superset";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
