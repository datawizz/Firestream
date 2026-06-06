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
  perSystem = { pkgs, lib, config, evalChart, ... }:
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

      # Phase F: firestream-kafka container defaults mostly match Bitnami's
      # `/opt/bitnami/kafka` + `/bitnami/kafka` layout (verified against
      # src/containers/firestream/kafka/options.nix). The only gap: the chart
      # does NOT mount `/opt/bitnami/kafka/tmp`, but the container's default
      # KAFKA_TMP_DIR points there. With `readOnlyRootFilesystem: true` on
      # both `prepare-config` init and main `kafka` pods, that write fails.
      # Redirect KAFKA_TMP_DIR + KAFKA_PID_FILE to `/tmp` (chart mounts an
      # emptyDir there for both init + main).
      #
      # All the other Bitnami-aligned paths (VOLUME=/bitnami/kafka,
      # DATA=/bitnami/kafka/data, CONF=/opt/bitnami/kafka/config,
      # CONF_FILE=/opt/bitnami/kafka/config/server.properties,
      # LOG=/opt/bitnami/kafka/logs) match the container defaults; setting
      # them again here is harmless and documents the chart contract.
      firestreamPathOverrides = [
        # PVC mount
        { name = "KAFKA_VOLUME_DIR"; value = "/bitnami/kafka"; }
        { name = "KAFKA_DATA_DIR";   value = "/bitnami/kafka/data"; }
        # Config (subPath mount for server.properties; rest of dir is read-only)
        { name = "KAFKA_CONF_DIR";  value = "/opt/bitnami/kafka/config"; }
        { name = "KAFKA_CONF_FILE"; value = "/opt/bitnami/kafka/config/server.properties"; }
        # Log dir (emptyDir mount)
        { name = "KAFKA_LOG_DIR"; value = "/opt/bitnami/kafka/logs"; }
        # Tmp dir / pid file — no chart mount at /opt/bitnami/kafka/tmp,
        # redirect to /tmp (mounted as emptyDir in both init + main).
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
        # is disabled in Bitnami's default values. Set extraEnvVars on both
        # so a values-override that flips controllerOnly=true (split mode)
        # still gets the path remap.
        config.kafka.controller.extraEnvVars = firestreamPathOverrides;
        config.kafka.broker.extraEnvVars = firestreamPathOverrides;
      };

      c = evalChart {
        name = "kafka";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
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
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
