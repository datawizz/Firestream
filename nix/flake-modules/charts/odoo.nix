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
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
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

      # Path de-branding: firestream-odoo now bakes its app tree at
      # /opt/firestream/odoo and its data/volume at /firestream/odoo (see
      # src/containers/firestream/odoo/options.nix + module.nix). The chart
      # templates were de-branded to mount the data PVC at /firestream/odoo and
      # the Secret at /opt/firestream/odoo/secrets — i.e. the chart mount now
      # EQUALS the container's baked ODOO_VOLUME_DIR/ODOO_DATA_DIR. No
      # extraEnvVars path-remap is needed (and any remap would re-introduce a
      # mount/baked mismatch), so the former firestreamPathOverrides is gone.

      # Phase 1 path-de-branding: the postgresql subchart templates + values now
      # mount/reference firestream paths directly, matching the firestream-postgresql
      # container's baked *_DIR vars. No subchart pg path-override extraEnvVars needed.
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

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders `bitnami/odoo`.
      packages.odoo-base-chart = baseChart {
        name = "odoo";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.odoo = c // { baseChart = config.packages.odoo-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.odoo.
      firestreamChartImages.odoo = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.odoo-base-chart;
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
