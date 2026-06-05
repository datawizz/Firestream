# Kafka chart flake-module (Phase 6b - Agent G3)
# Copyright Firestream. MIT License.
#
# Wires the kafka Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.kafka-chart                (deployable chart bundle; builds on darwin)
#   - firestreamCharts.kafka              (full evaluated chart result, for aggregate)
#   - firestreamChartImages.kafka         (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/redis.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# Image injection is NOT yet wired (no firestreamImages.kafka container in
# the registry has its image consumed here). The Bitnami chart ships four
# images (kafka, jmx-exporter, kubectl, os-shell); only the main `kafka`
# image would be injected (componentPath = [ "image" ]) once a Firestream
# container exists. For now, `_meta.containerRefs` stays empty and the
# bundled Bitnami images render unchanged; no
# `global.security.allowInsecureImages` flip needed.
#
# Subcharts: Kafka 4.x is pure KRaft and only depends on `common` (no
# zookeeper subchart). vendor-subcharts.nix handles the (trivial) common
# dependency.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/kafka;
      optionsPath = chartSrc + "/nix/default.nix";

      subcharts = [
        { name = "common"; }
      ];

      c = evalChart {
        name = "kafka";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
      };
    in
    {
      packages.kafka-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.kafka = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.kafka.
      firestreamChartImages.kafka = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "kafka";
          inherit chartSrc subcharts;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
