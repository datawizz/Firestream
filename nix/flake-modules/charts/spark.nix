# Spark chart flake-module (Phase 6b - Agent G4)
# Copyright Firestream. MIT License.
#
# Wires the spark Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.spark-chart                (deployable chart bundle; builds on darwin)
#   - firestreamCharts.spark              (full evaluated chart result, for aggregate)
#   - firestreamChartImages.spark         (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/kafka.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# The Bitnami spark chart is the standalone master/worker variant (NOT the
# Kubernetes spark-operator). Only `common` is vendored.
#
# Image injection is NOT yet wired (no firestreamImages.spark container in
# the registry has its image consumed here). The Bitnami chart ships a
# single `bitnami/spark` image; that image would be injected
# (componentPath = [ "image" ]) once a Firestream container exists. For
# now, `_meta.containerRefs` stays empty and the bundled Bitnami image
# renders unchanged; no `global.security.allowInsecureImages` flip needed.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/spark;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. Spark's Chart.yaml
      # lists `common` as its only dependency; the engine's vendor-subcharts.nix
      # handles the nested copy.
      subcharts = [
        { name = "common"; }
      ];

      c = evalChart {
        name = "spark";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.spark-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.spark = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.spark.
      firestreamChartImages.spark = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "spark";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
