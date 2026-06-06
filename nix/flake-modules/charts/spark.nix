# Spark chart flake-module (Phase 6b - Agent G4; Phase B image injection)
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
# Phase B image injection: re-verified that `src/charts/firestream/spark/values.yaml`
# uses ONE top-level `image:` block (lines 109-129) that is shared by both
# master and worker pods via the chart's `spark.image` template helper. So a
# SINGLE slot at componentPath = [ "image" ] is sufficient — no separate
# master.image / worker.image overrides needed.
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist.
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

      sparkImg =
        let
          imgEval = config.firestreamImages.spark.eval (_: {});
          imgCfg = imgEval.config.spark.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      # Phase F: firestream-spark container bakes SPARK_* dirs under /opt/spark
      # (see src/containers/firestream/spark/options.nix). The Bitnami chart
      # mounts emptyDirs at /opt/bitnami/spark/{conf,tmp,logs,work} and /tmp.
      # Remap every SPARK_*_DIR / SPARK_*_FILE to the chart's actual mounts.
      #
      # readOnlyRootFilesystem is enforced on master + worker, so any write
      # outside the explicit mounts fails. SPARK_DATA_DIR / SPARK_USER_JARS_DIR
      # have no chart mount — co-locate them under /opt/bitnami/spark/work
      # (an emptyDir present on both pods) so jar-staging + scratch data
      # land on writable storage.
      firestreamPathOverrides = [
        { name = "SPARK_CONF_DIR";  value = "/opt/bitnami/spark/conf"; }
        { name = "SPARK_CONF_FILE"; value = "/opt/bitnami/spark/conf/spark-defaults.conf"; }
        { name = "SPARK_LOG_DIR";   value = "/opt/bitnami/spark/logs"; }
        { name = "SPARK_TMP_DIR";   value = "/opt/bitnami/spark/tmp"; }
        { name = "SPARK_WORK_DIR";  value = "/opt/bitnami/spark/work"; }
        { name = "SPARK_DATA_DIR";  value = "/opt/bitnami/spark/work/data"; }
        { name = "SPARK_USER_JARS_DIR"; value = "/opt/bitnami/spark/work/jars"; }
      ];

      imageInjectionModule = { ... }: {
        config.spark._meta.containerRefs.spark = {
          inherit (sparkImg) registry repository tag;
          componentPath = [ "image" ];
        };
        config.spark.global.security.allowInsecureImages = true;
        # Bitnami spark splits master + worker into their own pod specs but
        # shares a single top-level `image:`. extraEnvVars is per-component.
        config.spark.master.extraEnvVars = firestreamPathOverrides;
        config.spark.worker.extraEnvVars = firestreamPathOverrides;
      };

      c = evalChart {
        name = "spark";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
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
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
