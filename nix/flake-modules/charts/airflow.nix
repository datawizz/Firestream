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
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
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
      #
      # Path-de-branding: the firestream-airflow container now bakes
      # AIRFLOW_HOME / AIRFLOW_*_DIR / *_FILE env vars under /opt/firestream/airflow
      # (see src/containers/firestream/airflow/options.nix), and the chart
      # templates now mount emptyDirs / PVCs / configmaps / subPaths at the same
      # /opt/firestream/airflow layout. Because container-baked defaults and the
      # chart mount layout AGREE, the old `extraEnvVars` path-override remap is no
      # longer required and has been removed.
      #
      # The postgresql + redis subchart templates + values likewise mount/reference
      # firestream paths directly, so no subchart path-override extraEnvVars needed.
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
        # The bundled Bitnami postgresql subchart defaults to preloading the
        # `pgaudit` extension, which the firestream-postgresql image does not
        # ship (server start fails with `could not access file "pgaudit"`).
        # Mirror the standalone postgresql overlay and clear it for the subchart.
        config.airflow.postgresql.postgresqlSharedPreloadLibraries = "";
      };

      # Phase E: make SeaweedFS the default local S3 backend for Airflow pods
      # (workers/scheduler run etl_lib + submit Spark jobs). etl_lib's Spark
      # client reads S3_LOCAL_* env vars and sets spark.hadoop.fs.s3a.*
      # (path-style, ssl disabled) — data-driven, no Python change.
      #
      # We inject the five vars as PLAIN literal values (NOT secretKeyRef): the
      # seaweedfs Secret lives in the `seaweedfs` namespace and a secretKeyRef
      # cannot read a Secret from another namespace. These are deterministic dev
      # credentials for a single-node, not-internet-exposed store, so literal
      # injection is robust and simple. (Per-tenant secret replication for the
      # hosted/cloud tier is a documented follow-on.)
      #
      # The Bitnami airflow chart exposes a top-level `extraEnvVars` that is
      # applied to ALL Airflow pods (web/scheduler/worker/dagProcessor/
      # triggerer) — see values.yaml line 498. This is currently empty (the old
      # AIRFLOW_*_DIR path remaps were removed), so there is nothing to clobber;
      # a consumer's own `userMod` extraEnvVars would concatenate via the NixOS
      # module list-merge.
      s3LocalEnvVars = [
        { name = "S3_LOCAL_ENDPOINT_URL"; value = "http://seaweedfs-all-in-one.seaweedfs.svc.cluster.local:8333"; }
        { name = "S3_LOCAL_ACCESS_KEY_ID"; value = "firestream"; }
        { name = "S3_LOCAL_SECRET_ACCESS_KEY"; value = "firestream-secret"; }
        { name = "S3_LOCAL_BUCKET_NAME"; value = "firestream"; }
        { name = "S3_LOCAL_DEFAULT_REGION"; value = "us-east-1"; }
      ];

      s3EnvInjectionModule = { ... }: {
        config.airflow.extraEnvVars = s3LocalEnvVars;
      };

      c = evalChart {
        name = "airflow";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule s3EnvInjectionModule ];
      };
    in
    {
      packages.airflow-chart = c.chartBundle;

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders `bitnami/airflow`.
      packages.airflow-base-chart = baseChart {
        name = "airflow";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.airflow = c // { baseChart = config.packages.airflow-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.airflow.
      firestreamChartImages.airflow = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.airflow-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "airflow";
          inherit chartSrc subcharts;
          # Preserve the image + S3 injection in user re-evals so consumer
          # overrides still receive the registry-derived image triple and the
          # default SeaweedFS S3 env vars by default.
          modules = [ optionsPath imageInjectionModule s3EnvInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
