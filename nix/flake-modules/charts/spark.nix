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
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
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

      # Phase 1 path-de-branding: the firestream-spark container now bakes its
      # SPARK_* dirs under /opt/firestream/spark and /firestream/spark (see
      # src/containers/firestream/spark/options.nix), and the chart templates
      # mount their emptyDirs at the matching /opt/firestream/spark/{conf,tmp,
      # logs,work} paths. Container bake == chart mount, so the per-component
      # SPARK_*_DIR extraEnvVars remaps are no longer needed and were removed.
      imageInjectionModule = { ... }: {
        config.spark._meta.containerRefs.spark = {
          inherit (sparkImg) registry repository tag;
          componentPath = [ "image" ];
        };
        config.spark.global.security.allowInsecureImages = true;
      };

      # Phase E: make SeaweedFS the default local S3 backend. etl_lib's Spark
      # client (src/lib/python/etl-lib/src/etl_lib/services/spark/client.py)
      # reads S3_LOCAL_* env vars and sets spark.hadoop.fs.s3a.* (path-style,
      # ssl disabled) — purely data-driven, so no Python change is needed. We
      # inject the five env vars as PLAIN literal values (NOT secretKeyRef):
      # the seaweedfs Secret lives in the `seaweedfs` namespace and a
      # secretKeyRef cannot cross namespaces. Since these are deterministic dev
      # credentials for a single-node store that is not exposed to the internet,
      # literal injection is the robust, simple choice. (Per-tenant secret
      # replication for the hosted/cloud tier is a documented follow-on.)
      #
      # Injected into BOTH master and worker pods (the spark-submit driver may
      # run on a worker in cluster mode; over-injection of these vars is
      # harmless). These set master/worker.extraEnvVars, which currently carry
      # no Firestream values (the old SPARK_*_DIR path remaps were removed), so
      # there is nothing to clobber; if a consumer adds its own extraEnvVars via
      # the re-eval `userMod`, the NixOS module system concatenates the lists.
      s3LocalEnvVars = [
        { name = "S3_LOCAL_ENDPOINT_URL"; value = "http://seaweedfs-all-in-one.seaweedfs.svc.cluster.local:8333"; }
        { name = "S3_LOCAL_ACCESS_KEY_ID"; value = "firestream"; }
        { name = "S3_LOCAL_SECRET_ACCESS_KEY"; value = "firestream-secret"; }
        { name = "S3_LOCAL_BUCKET_NAME"; value = "firestream"; }
        { name = "S3_LOCAL_DEFAULT_REGION"; value = "us-east-1"; }
      ];

      s3EnvInjectionModule = { ... }: {
        config.spark.master.extraEnvVars = s3LocalEnvVars;
        config.spark.worker.extraEnvVars = s3LocalEnvVars;
      };

      c = evalChart {
        name = "spark";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule s3EnvInjectionModule ];
      };
    in
    {
      packages.spark-chart = c.chartBundle;

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders `bitnami/spark`.
      packages.spark-base-chart = baseChart {
        name = "spark";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.spark = c // { baseChart = config.packages.spark-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.spark.
      firestreamChartImages.spark = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.spark-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "spark";
          inherit chartSrc subcharts;
          modules = [ optionsPath imageInjectionModule s3EnvInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
