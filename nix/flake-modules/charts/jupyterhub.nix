# JupyterHub chart flake-module (Phase 6b - Agent G5)
# Copyright Firestream. MIT License.
#
# Wires the JupyterHub Helm chart through the options-driven evalChart
# entrypoint and contributes:
#   - packages.jupyterhub-chart       (deployable chart bundle; builds on darwin)
#   - firestreamCharts.jupyterhub     (full evaluated chart result, for aggregate)
#   - firestreamChartImages.jupyterhub (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/spark.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# The Bitnami JupyterHub chart bundles `common` and `postgresql` as vendored
# subcharts (Chart.yaml lists both). vendor-subcharts.nix consumes the
# in-repo Bitnami fork at src/charts/bitnami/.
#
# Image injection is NOT yet wired (no firestreamImages.jupyterhub container
# in the registry has its image consumed here). The Bitnami chart ships four
# images (hub, proxy, singleuser, os-shell auxiliary); those would each be
# injected (componentPath = [ "hub" "image" ] / [ "proxy" "image" ] /
# [ "singleuser" "image" ] / [ "auxiliaryImage" ]) once Firestream containers
# exist. For now, `_meta.containerRefs` stays empty and the bundled Bitnami
# images render unchanged; no `global.security.allowInsecureImages` flip
# needed.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/jupyterhub;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. JupyterHub's
      # Chart.yaml lists `postgresql` (16.x.x) and `common` (2.x.x). The
      # engine's vendor-subcharts.nix handles the nested copy.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
      ];

      c = evalChart {
        name = "jupyterhub";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.jupyterhub-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.jupyterhub = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.jupyterhub.
      firestreamChartImages.jupyterhub = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "jupyterhub";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
