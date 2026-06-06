# PostgreSQL chart flake-module (Phase 6b - Agent G1; Phase B image injection)
# Copyright Firestream. MIT License.
#
# Wires the postgresql Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.postgresql-chart          (deployable chart bundle; builds on darwin)
#   - firestreamCharts.postgresql        (full evaluated chart result, for aggregate)
#   - firestreamChartImages.postgresql   (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/airflow.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# Phase B image injection: an `imageInjectionModule` is appended to the chart's
# modules list so its writes to `_meta.containerRefs` merge into the chart eval.
# It sources the postgresql container's image triple from
# `config.firestreamImages.postgresql.eval (_: {})` and projects it into one
# slot named "postgresql" with `componentPath = [ "image" ]` (the chart's
# top-level `image:` block in values.yaml — Bitnami postgresql is a single-
# image chart). `global.security.allowInsecureImages` is flipped to bypass
# Bitnami's NOTES.txt whitelist (which rejects the `firestream-postgresql`
# repo name).
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/postgresql;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. PostgreSQL's Chart.yaml
      # lists `common` as its only dependency; the engine's vendor-subcharts.nix
      # handles the nested copy.
      subcharts = [
        { name = "common"; }
      ];

      # Source the postgresql container's image triple by re-evaluating the
      # container with no overrides. Reading `.config.postgresql.image` only
      # forces option evaluation - it does NOT force the (Linux-only) docker
      # image build, so this works on Darwin too.
      pgImg =
        let
          imgEval = config.firestreamImages.postgresql.eval (_: {});
          imgCfg = imgEval.config.postgresql.image;
        in {
          registry = imgCfg.registry;          # may be null
          repository = imgCfg.repository;      # always non-null per container schema
          tag = imgEval.imageTag;              # falls back to version when image.tag is null
        };

      # firestream-postgresql container bakes its FHS under
      # /opt/firestream/postgresql/{bin,conf.default,conf,tmp,logs} and
      # /firestream/postgresql/{data,conf} — those nested store paths are
      # read-only at runtime (nix store) or simply not where the Bitnami
      # chart provides PVC/emptyDir mounts. The Bitnami chart only mounts
      # writable volumes at:
      #   /bitnami/postgresql        (PVC, persistent data)
      #   /opt/bitnami/postgresql/conf   (emptyDir, writable)
      #   /opt/bitnami/postgresql/tmp    (emptyDir, writable)
      #   /opt/bitnami/postgresql/secrets (Secret mount, read-only)
      #
      # Bridging without rebuilding the container: redirect every
      # runtime-writable POSTGRESQL_* dir to a path the chart actually mounts.
      # POSTGRESQL_DEFAULT_CONF_DIR stays at /opt/firestream/postgresql/conf.default
      # because that's where the image-baked template files live (used by
      # postgresql_create_config as a seed).
      #
      # NOTE: env-defaults.sh derives CONF_DIR/TMP_DIR/LOG_DIR/DATA_DIR/...
      # from BASE_DIR and VOLUME_DIR, but only via `${VAR:-default}` — which
      # respects whatever the image ENV already set. The container's module.nix
      # bakes every *_DIR var as a literal, so we must override each *_DIR var
      # the chart cares about explicitly (overriding only BASE_DIR/VOLUME_DIR
      # is NOT sufficient).
      firestreamPathOverrides = [
        # Data / volume dirs (PVC mounted at /bitnami/postgresql)
        { name = "POSTGRESQL_VOLUME_DIR"; value = "/bitnami/postgresql"; }
        { name = "POSTGRESQL_DATA_DIR";   value = "/bitnami/postgresql/data"; }
        { name = "POSTGRESQL_MOUNTED_CONF_DIR"; value = "/bitnami/postgresql/conf"; }
        # Config dir (emptyDir at /opt/bitnami/postgresql/conf, writable)
        { name = "POSTGRESQL_CONF_DIR";  value = "/opt/bitnami/postgresql/conf"; }
        { name = "POSTGRESQL_CONF_FILE"; value = "/opt/bitnami/postgresql/conf/postgresql.conf"; }
        { name = "POSTGRESQL_PGHBA_FILE"; value = "/opt/bitnami/postgresql/conf/pg_hba.conf"; }
        { name = "POSTGRESQL_REPLICATION_PASSFILE_PATH"; value = "/opt/bitnami/postgresql/conf/.pgpass"; }
        # Tmp/PID dir (emptyDir at /opt/bitnami/postgresql/tmp, writable)
        { name = "POSTGRESQL_TMP_DIR";   value = "/opt/bitnami/postgresql/tmp"; }
        { name = "POSTGRESQL_PID_FILE";  value = "/opt/bitnami/postgresql/tmp/postgresql.pid"; }
        # Log dir (no dedicated mount; co-locate under PVC so logs persist)
        { name = "POSTGRESQL_LOG_DIR";   value = "/bitnami/postgresql/logs"; }
        { name = "POSTGRESQL_LOG_FILE";  value = "/bitnami/postgresql/logs/postgresql.log"; }
      ];

      imageInjectionModule = { ... }: {
        config.postgresql._meta.containerRefs.postgresql = {
          inherit (pgImg) registry repository tag;
          componentPath = [ "image" ];
        };
        config.postgresql.global.security.allowInsecureImages = true;
        config.postgresql.primary.extraEnvVars = firestreamPathOverrides;
        # The Bitnami chart defaults to preloading the `pgaudit` extension;
        # firestream-postgresql ships without it. Disable the preload at the
        # chart level so postgres doesn't refuse to start with
        # "could not access file pgaudit: No such file or directory".
        config.postgresql.postgresqlSharedPreloadLibraries = "";
      };

      c = evalChart {
        name = "postgresql";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.postgresql-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.postgresql = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.postgresql.
      firestreamChartImages.postgresql = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "postgresql";
          inherit chartSrc subcharts;
          # Preserve the image injection in user re-evals so consumer overrides
          # still receive the registry-derived image triple by default.
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
