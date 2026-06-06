# Airflow chart flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the airflow Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.airflow-chart          (deployable chart bundle; builds on darwin)
#   - firestreamCharts.airflow        (full evaluated chart result, for aggregate)
#   - firestreamChartImages.airflow   (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/containers/airflow.nix. Unlike containers, charts are
# not Docker images, so there is NO isLinux gate — `helm template` runs in the
# build sandbox on every platform.
#
# Phase 2 (Agent B): an `imageInjectionModule` is attached AFTER the user's
# optionsPath module so its writes to `_meta.containerRefs` are merged into the
# chart eval. It sources the airflow container's image triple from
# `config.firestreamImages.airflow.eval (_: {})` (which yields a fresh eval
# of the container module with no overrides) and projects it into one slot
# named "airflow" with `componentPath = [ "image" ]` - the only Bitnami-airflow
# image slot the chart actually consumes (verified against
# src/charts/firestream/airflow/values.yaml: the chart has a single top-level
# `image:` plus a `metrics.image:` for the StatsD exporter, and components
# web/scheduler/worker/dagProcessor/triggerer all share the top-level image
# via the `airflow.image` helper template).
#
# Phase B (image-injection sweep): adds two MORE slots for the bundled
# postgresql and redis subcharts so the rendered chart pulls
# `firestream-postgresql:17` + `firestream-redis:8` instead of
# `bitnami/postgresql` + `bitnami/redis` for the bundled deployment. The
# main `airflow` slot is unchanged.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/airflow;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. `common` is nested under
      # postgresql/redis too; the engine's vendor-subcharts handles that.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
        { name = "redis"; }
      ];

      # Source the airflow container's image triple by re-evaluating the
      # container with no overrides. Reading `.config.airflow.image` only forces
      # option evaluation - it does NOT force the (Linux-only) docker image
      # build, so this works on Darwin too. (Mirrors the pattern used by
      # compose.nix.)
      airflowImg =
        let
          imgEval = config.firestreamImages.airflow.eval (_: {});
          imgCfg = imgEval.config.airflow.image;
        in {
          registry = imgCfg.registry;          # may be null
          repository = imgCfg.repository;      # always non-null per container schema
          tag = imgEval.imageTag;              # falls back to version when image.tag is null
        };

      # Phase B: subchart image triples for the bundled postgresql + redis.
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

      # Override module: populate `_meta.containerRefs` with one slot pointing
      # at the chart's shared top-level `image:`. Layering this AFTER optionsPath
      # in `evalChart`'s modules list ensures it merges into the same
      # `config.airflow._meta.containerRefs` attrset declared by eval-chart's
      # standard schema.
      #
      # We also flip `global.security.allowInsecureImages` to true: the Bitnami
      # chart ships a NOTES.txt assertion that REFUSES to render when the
      # image repository does not match a curated whitelist (`bitnami/airflow`,
      # `bitnamilegacy/airflow`, ...). Since the Firestream container produces
      # `firestream-airflow` by design, we must opt out of that verification.
      # This is the documented Bitnami escape hatch
      # (https://github.com/bitnami/charts/issues/30850) and applies ONLY when
      # the chart is consumed with our injected image.
      # Phase F: firestream-airflow bakes AIRFLOW_*_DIR / *_FILE env vars
      # under /opt/airflow (see src/containers/firestream/airflow/options.nix).
      # The Bitnami chart mounts emptyDirs at /opt/bitnami/airflow/{logs,tmp,
      # nss-wrapper,secrets,venv} and subPath mounts for airflow.cfg /
      # webserver_config.py / airflow.db. /emptydir is the parent for the
      # init container's bootstrap workspace. readOnlyRootFilesystem is
      # enforced on all components.
      #
      # Remap every path var to the chart's actual mount layout. Since the
      # chart's `extraEnvVars` lives at the top level of values.yaml and
      # applies to ALL pods (web/scheduler/worker/dagProcessor/triggerer +
      # init/wait jobs), one set covers the whole deployment.
      firestreamPathOverrides = [
        { name = "AIRFLOW_CONF_FILE";           value = "/opt/bitnami/airflow/airflow.cfg"; }
        { name = "AIRFLOW_WEBSERVER_CONF_FILE"; value = "/opt/bitnami/airflow/webserver_config.py"; }
        { name = "AIRFLOW_LOGS_DIR";            value = "/opt/bitnami/airflow/logs"; }
        { name = "AIRFLOW_SCHEDULER_LOGS_DIR";  value = "/opt/bitnami/airflow/logs/scheduler"; }
        { name = "AIRFLOW_TMP_DIR";             value = "/opt/bitnami/airflow/tmp"; }
        { name = "AIRFLOW_DAGS_DIR";            value = "/opt/bitnami/airflow/dags"; }
        { name = "AIRFLOW_PLUGINS_DIR";         value = "/opt/bitnami/airflow/plugins"; }
      ];

      imageInjectionModule = { ... }: {
        config.airflow._meta.containerRefs = {
          airflow = {
            inherit (airflowImg) registry repository tag;
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
        config.airflow.global.security.allowInsecureImages = true;
        # Top-level airflow.extraEnvVars applies to every airflow component
        # pod (web, scheduler, worker, dagProcessor, triggerer) AND the
        # init/setup-db jobs. Single point of truth.
        config.airflow.extraEnvVars = firestreamPathOverrides;
      };

      c = evalChart {
        name = "airflow";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.airflow-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.airflow = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.airflow.
      firestreamChartImages.airflow = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "airflow";
          inherit chartSrc subcharts;
          # Preserve the image injection in user re-evals so consumer overrides
          # still receive the registry-derived image triple by default.
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
