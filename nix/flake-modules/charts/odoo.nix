# Odoo chart flake-module (Phase 6b - Agent G7; Phase B image injection)
# Copyright Firestream. MIT License.
#
# Wires the Odoo Helm chart through the options-driven evalChart
# entrypoint and contributes:
#   - packages.odoo-chart            (deployable chart bundle; builds on darwin)
#   - firestreamCharts.odoo          (full evaluated chart result, for aggregate)
#   - firestreamChartImages.odoo     (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/superset.nix. No isLinux gate —
# `helm template` runs in the build sandbox on every platform.
#
# The Bitnami Odoo chart bundles `common` + `postgresql` as vendored
# subcharts (Chart.yaml v28.2.7 lists both). Odoo does NOT use redis,
# so the subchart list is shorter than superset's. vendor-subcharts.nix
# consumes the in-repo Bitnami fork at src/charts/bitnami/. The
# recursive vendoring handles postgresql's own `common` dependency.
#
# Phase B image injection: Bitnami Odoo ships ONE main image (bitnami/odoo,
# top-level `image:` shared by app + init containers) plus the bundled
# postgresql subchart. We wire both with firestream-* containers.
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/odoo;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. Odoo's
      # Chart.yaml lists `postgresql` (16.x.x) and `common` (2.x.x).
      # NO redis (unlike superset). The engine's vendor-subcharts.nix
      # handles the nested copy AND postgresql's own `common`
      # dependency.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
      ];

      odooImg =
        let
          imgEval = config.firestreamImages.odoo.eval (_: {});
          imgCfg = imgEval.config.odoo.image;
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

      # Phase F: firestream-odoo bakes ODOO_*_DIR under /opt/odoo by default
      # (see src/containers/firestream/odoo/options.nix). The Bitnami chart
      # mounts the PVC at /bitnami/odoo and a Secret at /opt/bitnami/odoo/secrets.
      # readOnlyRootFilesystem is false on the main odoo container — so the
      # only paths that strictly NEED remapping are the volume/data dirs that
      # must land on the PVC. We remap conf/logs/tmp/addons to /bitnami/odoo/...
      # too so they persist + share the same writable mount.
      firestreamPathOverrides = [
        # PVC mount (top-level)
        { name = "ODOO_VOLUME_DIR"; value = "/bitnami/odoo"; }
        { name = "ODOO_DATA_DIR";   value = "/bitnami/odoo/data"; }
        { name = "ODOO_ADDONS_DIR"; value = "/bitnami/odoo/addons"; }
        { name = "ODOO_CONF_DIR";   value = "/bitnami/odoo/conf"; }
        { name = "ODOO_CONF_FILE";  value = "/bitnami/odoo/conf/odoo.conf"; }
        { name = "ODOO_LOGS_DIR";   value = "/bitnami/odoo/log"; }
        { name = "ODOO_LOG_FILE";   value = "/bitnami/odoo/log/odoo-server.log"; }
        { name = "ODOO_TMP_DIR";    value = "/bitnami/odoo/tmp"; }
        { name = "ODOO_PID_FILE";   value = "/bitnami/odoo/tmp/odoo.pid"; }
      ];

      # Phase G fix: subchart postgres also needs path remap so its
      # /opt/firestream/postgresql/conf (baked, read-only when
      # readOnlyRootFilesystem=true) is redirected to the chart-mounted
      # emptyDir at /opt/bitnami/postgresql/conf. Same set as the
      # standalone postgresql chart (nix/flake-modules/charts/postgresql.nix:72-88).
      pgFirestreamPathOverrides = [
        { name = "POSTGRESQL_VOLUME_DIR"; value = "/bitnami/postgresql"; }
        { name = "POSTGRESQL_DATA_DIR";   value = "/bitnami/postgresql/data"; }
        { name = "POSTGRESQL_MOUNTED_CONF_DIR"; value = "/bitnami/postgresql/conf"; }
        { name = "POSTGRESQL_CONF_DIR";  value = "/opt/bitnami/postgresql/conf"; }
        { name = "POSTGRESQL_CONF_FILE"; value = "/opt/bitnami/postgresql/conf/postgresql.conf"; }
        { name = "POSTGRESQL_PGHBA_FILE"; value = "/opt/bitnami/postgresql/conf/pg_hba.conf"; }
        { name = "POSTGRESQL_REPLICATION_PASSFILE_PATH"; value = "/opt/bitnami/postgresql/conf/.pgpass"; }
        { name = "POSTGRESQL_TMP_DIR";   value = "/opt/bitnami/postgresql/tmp"; }
        { name = "POSTGRESQL_PID_FILE";  value = "/opt/bitnami/postgresql/tmp/postgresql.pid"; }
        { name = "POSTGRESQL_LOG_DIR";   value = "/bitnami/postgresql/logs"; }
        { name = "POSTGRESQL_LOG_FILE";  value = "/bitnami/postgresql/logs/postgresql.log"; }
      ];

      imageInjectionModule = { ... }: {
        config.odoo._meta.containerRefs = {
          odoo = {
            inherit (odooImg) registry repository tag;
            componentPath = [ "image" ];
          };
          postgresql = {
            inherit (pgImg) registry repository tag;
            componentPath = [ "postgresql" "image" ];
          };
        };
        config.odoo.global.security.allowInsecureImages = true;
        # Bitnami odoo is single-component (no master/worker split).
        # extraEnvVars sits at the top level of values.yaml.
        config.odoo.extraEnvVars = firestreamPathOverrides;
        # Subchart postgres: inject path overrides on the subchart's
        # primary.extraEnvVars (mirrors standalone postgresql chart).
        config.odoo.postgresql.primary.extraEnvVars = pgFirestreamPathOverrides;
        config.odoo.postgresql.postgresqlSharedPreloadLibraries = "";
      };

      c = evalChart {
        name = "odoo";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.odoo-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.odoo = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.odoo.
      firestreamChartImages.odoo = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "odoo";
          inherit chartSrc subcharts;
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
