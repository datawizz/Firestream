# Odoo chart flake-module (Phase 6b - Agent G7)
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
# Image injection is NOT yet wired (no firestreamImages.odoo container
# in the registry has its image consumed here). The Bitnami chart ships
# ONE main image (bitnami/odoo); that would be injected
# (componentPath = [ "image" ]) once a Firestream container exists.
# For now, `_meta.containerRefs` stays empty and the bundled Bitnami
# image renders unchanged; no `global.security.allowInsecureImages`
# flip needed.
#
# Milestone marker: odoo is the EIGHTH and final chart wired in this
# phase. After this lands, `firestreamCharts` has full coverage of the
# `firestreamStacks.dev` chart set (airflow, jupyterhub, kafka, odoo,
# postgresql, redis, spark, superset).
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

      c = evalChart {
        name = "odoo";
        inherit chartSrc subcharts;
        modules = [ optionsPath ];
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
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
