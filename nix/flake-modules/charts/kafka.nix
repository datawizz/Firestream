# Kafka chart flake-module (Phase 6b - Agent G3; Phase B image injection)
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
# Phase B image injection: the Bitnami chart ships four images (kafka,
# jmx-exporter, kubectl, os-shell). Only the main `kafka` image is wired here
# (componentPath = [ "image" ] — the chart's top-level `image:` block shared by
# the controller/broker pods). The jmx-exporter / kubectl / os-shell sidecars
# remain at their bundled Bitnami refs because Firestream has no replacement
# container for them yet — but they are NOT pulled in the default chart values
# (metrics.kafka.enabled = false, externalAccess.autoDiscovery = false, default
# init-containers off), so they do not appear in `rendered.yaml`.
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist.
#
# Subcharts: Kafka 4.x is pure KRaft and only depends on `common` (no
# zookeeper subchart). vendor-subcharts.nix handles the (trivial) common
# dependency.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/kafka;
      optionsPath = chartSrc + "/nix/default.nix";

      subcharts = [
        { name = "common"; }
      ];

      kafkaImg =
        let
          imgEval = config.firestreamImages.kafka.eval (_: {});
          imgCfg = imgEval.config.kafka.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      # Phase 3 de-brand: the firestream-kafka container now bakes its
      # functional paths under `/opt/firestream/kafka` + `/firestream/kafka`
      # (src/containers/firestream/kafka/{options,module}.nix) and the chart
      # templates/values mount the exact same paths, so no path-remap
      # extraEnvVars are needed any more — the binding + both injections were
      # removed. KAFKA_TMP_DIR / KAFKA_PID_FILE still point at the chart's
      # `/tmp` emptyDir (the chart does not mount the container's tmp dir and
      # the pods run readOnlyRootFilesystem) — these are functional, not
      # de-branding, redirects.
      firestreamTmpOverrides = [
        { name = "KAFKA_TMP_DIR"; value = "/tmp"; }
        { name = "KAFKA_PID_FILE"; value = "/tmp/kafka.pid"; }
      ];

      imageInjectionModule = { ... }: {
        config.kafka._meta.containerRefs.kafka = {
          inherit (kafkaImg) registry repository tag;
          componentPath = [ "image" ];
        };
        config.kafka.global.security.allowInsecureImages = true;
        # Kafka 4.x KRaft uses combined controller+broker pods by default.
        # The `controller` StatefulSet runs both roles; `broker` StatefulSet
        # is disabled in Bitnami's default values. Set the tmp redirect on
        # both so a values-override that flips controllerOnly=true (split
        # mode) still gets it.
        config.kafka.controller.extraEnvVars = firestreamTmpOverrides;
        config.kafka.broker.extraEnvVars = firestreamTmpOverrides;
      };

      c = evalChart {
        name = "kafka";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.kafka-chart = c.chartBundle;

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders `bitnami/kafka`.
      packages.kafka-base-chart = baseChart {
        name = "kafka";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.kafka = c // { baseChart = config.packages.kafka-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.kafka.
      firestreamChartImages.kafka = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.kafka-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "kafka";
          inherit chartSrc subcharts;
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
