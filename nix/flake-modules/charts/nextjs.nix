# Next.js chart flake-module
# Copyright Firestream. MIT License.
#
# Wires the net-new nextjs Helm chart through the options-driven evalChart
# entrypoint and contributes:
#   - packages.nextjs-chart           (deployable chart bundle; builds on darwin)
#   - packages.nextjs-base-chart      (un-overlaid chart, native defaults)
#   - firestreamCharts.nextjs         (full evaluated chart result, for aggregate)
#   - firestreamChartImages.nextjs    (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/odoo.nix. The chart bundles `common` +
# `postgresql` as vendored subcharts and injects firestream-* images for BOTH
# the nextjs Deployment (top-level `image`) and the bundled postgresql subchart
# (`postgresql.image`), so the database runs the firestream-postgresql image.
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/nextjs;
      optionsPath = chartSrc + "/nix/default.nix";

      # Vendored from the in-repo Bitnami fork. The engine's vendor-subcharts.nix
      # handles postgresql's own `common` dependency.
      subcharts = [
        { name = "common"; }
        { name = "postgresql"; }
      ];

      # Next.js container image triple (re-eval without overrides). `.eval`
      # reads only config/imageTag — safe on darwin where dockerImage is gated.
      nextjsImg =
        let
          imgEval = config.firestreamImages.nextjs.eval (_: { });
          imgCfg = imgEval.config.nextjs.image;
        in
        {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      pgImg =
        let
          imgEval = config.firestreamImages.postgresql.eval (_: { });
          imgCfg = imgEval.config.postgresql.image;
        in
        {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      imageInjectionModule = { ... }: {
        config.nextjs._meta.containerRefs = {
          nextjs = {
            inherit (nextjsImg) registry repository tag;
            # Flat chart shape: the main image lives at top-level `image:`.
            componentPath = [ "image" ];
          };
          postgresql = {
            inherit (pgImg) registry repository tag;
            componentPath = [ "postgresql" "image" ];
          };
        };
        config.nextjs.global.security.allowInsecureImages = true;
        # The firestream-postgresql image does not ship the pgaudit extension,
        # but the Bitnami postgresql chart defaults shared_preload_libraries to
        # "pgaudit" — which makes the server FATAL on startup ("could not access
        # file pgaudit") and crashloop. Blank it out, exactly as the standalone
        # postgresql and odoo flake-modules do.
        config.nextjs.postgresql.postgresqlSharedPreloadLibraries = "";
      };

      c = evalChart {
        name = "nextjs";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.nextjs-chart = c.chartBundle;

      packages.nextjs-base-chart = baseChart {
        name = "nextjs";
        inherit chartSrc subcharts;
      };

      firestreamCharts.nextjs = c // { baseChart = config.packages.nextjs-base-chart; };

      firestreamChartImages.nextjs = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.nextjs-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "nextjs";
          inherit chartSrc subcharts;
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
