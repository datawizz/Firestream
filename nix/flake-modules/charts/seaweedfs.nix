# SeaweedFS chart flake-module (Phase C — object store).
# Copyright Firestream. MIT License.
#
# Wires the seaweedfs Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.seaweedfs-chart          (deployable chart bundle; builds on darwin)
#   - packages.seaweedfs-base-chart     (plain forked chart, native defaults)
#   - firestreamCharts.seaweedfs        (full evaluated chart result, for aggregate)
#   - firestreamChartImages.seaweedfs   (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/postgresql.nix. No isLinux gate — `helm
# template` runs in the build sandbox on every platform.
#
# SeaweedFS is NOT a Bitnami chart: there is no `global.security.allowInsecureImages`
# guard to flip, and the chart has no chart dependencies (subcharts = []).
#
# Image injection: an `imageInjectionModule` is appended to the chart's modules
# list so its writes to `_meta.containerRefs` merge into the chart eval. It
# sources the seaweedfs container's image triple from
# `config.firestreamImages.seaweedfs.eval (_: {})` and projects it into one slot
# named "seaweedfs" with `componentPath = [ "image" ]` (the chart's top-level
# `image:` block, read by the `seaweedfs.image` helper in
# templates/shared/_helpers.tpl).
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, baseChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/seaweedfs;
      optionsPath = chartSrc + "/nix/default.nix";

      # SeaweedFS has no chart dependencies.
      subcharts = [ ];

      # Source the seaweedfs container's image triple by re-evaluating the
      # container with no overrides. Reading `.config.seaweedfs.image` only
      # forces option evaluation — it does NOT force the (Linux-only) docker
      # image build, so this works on Darwin too.
      img =
        let
          imgEval = config.firestreamImages.seaweedfs.eval (_: {});
          imgCfg = imgEval.config.seaweedfs.image;
        in {
          registry = imgCfg.registry;          # may be null
          repository = imgCfg.repository;      # always non-null per container schema
          tag = imgEval.imageTag;              # falls back to version when image.tag is null
        };

      imageInjectionModule = { ... }: {
        config.seaweedfs._meta.containerRefs.seaweedfs = {
          inherit (img) registry repository tag;
          componentPath = [ "image" ];
        };
      };

      c = evalChart {
        name = "seaweedfs";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.seaweedfs-chart = c.chartBundle;

      # Base (un-overlaid) chart: chart's OWN native defaults, no Firestream
      # values overlay / no image injection. Renders chrislusf/seaweedfs.
      packages.seaweedfs-base-chart = baseChart {
        name = "seaweedfs";
        inherit chartSrc subcharts;
      };

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.seaweedfs = c // { baseChart = config.packages.seaweedfs-base-chart; };

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.seaweedfs.
      firestreamChartImages.seaweedfs = {
        chartBundle = c.chartBundle;
        baseChart = config.packages.seaweedfs-base-chart;
        render = c.render;
        eval = userMod: evalChart {
          name = "seaweedfs";
          inherit chartSrc subcharts;
          # Preserve the image injection in user re-evals so consumer overrides
          # still receive the registry-derived image triple by default.
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
