# Superset chart flake-module (Phase 6b - Agent G6; Phase B image injection)
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
# Phase B image injection: Bitnami Superset ships ONE main image
# (bitnami/superset — shared across web/worker/beat/flower/init via the chart's
# `image` template helper), plus the bundled postgresql and redis subchart
# images. We wire all three with firestream-* containers.
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist.
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

      supersetImg =
        let
          imgEval = config.firestreamImages.superset.eval (_: {});
          imgCfg = imgEval.config.superset.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      pgImg =
        let
          imgEval = config.firestreamImages.postgresql.eval (_: {});
          imgCfg = imgEval.config.postgresql.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      redisImg =
        let
          imgEval = config.firestreamImages.redis.eval (_: {});
          imgCfg = imgEval.config.redis.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      # Phase F: firestream-superset bakes SUPERSET_* dirs under /opt/superset
      # (see src/containers/firestream/superset/5/options.nix). The Bitnami
      # chart mounts an emptyDir at /opt/bitnami/superset/superset_home
      # (subPath superset-home) and /tmp; a Secret at
      # /opt/bitnami/superset/secrets. readOnlyRootFilesystem is enforced.
      # Co-locate logs/config under the superset_home emptyDir so they share
      # writable storage; redirect TMP_DIR to /tmp.
      firestreamPathOverrides = [
        { name = "SUPERSET_HOME_DIR";    value = "/opt/bitnami/superset/superset_home"; }
        { name = "SUPERSET_CONFIG_PATH"; value = "/opt/bitnami/superset/superset_home/superset_config.py"; }
        { name = "SUPERSET_LOG_DIR";     value = "/opt/bitnami/superset/superset_home/logs"; }
        { name = "SUPERSET_TMP_DIR";     value = "/tmp"; }
      ];

      imageInjectionModule = { ... }: {
        config.superset._meta.containerRefs = {
          superset = {
            inherit (supersetImg) registry repository tag;
            componentPath = [ "image" ];
          };
          postgresql = {
            inherit (pgImg) registry repository tag;
            componentPath = [ "postgresql" "image" ];
          };
          redis = {
            inherit (redisImg) registry repository tag;
            componentPath = [ "redis" "image" ];
          };
        };
        config.superset.global.security.allowInsecureImages = true;
        # Superset chart components: web (Flask webserver), worker (Celery),
        # init (DB bootstrap job). Apply remap to all three so init/migrate
        # writes go to the right mount and workers can read the same config.
        config.superset.web.extraEnvVars = firestreamPathOverrides;
        config.superset.worker.extraEnvVars = firestreamPathOverrides;
        config.superset.init.extraEnvVars = firestreamPathOverrides;
      };

      c = evalChart {
        name = "superset";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
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
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
