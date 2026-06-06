# Redis chart flake-module (Phase 6b - Agent G2; Phase B image injection)
# Copyright Firestream. MIT License.
#
# Wires the redis Helm chart through the options-driven evalChart entrypoint
# and contributes:
#   - packages.redis-chart                (deployable chart bundle; builds on darwin)
#   - firestreamCharts.redis              (full evaluated chart result, for aggregate)
#   - firestreamChartImages.redis         (consumer override API, for flake.lib.charts)
#
# Mirrors nix/flake-modules/charts/postgresql.nix. No isLinux gate — `helm template`
# runs in the build sandbox on every platform.
#
# Phase B image injection: an `imageInjectionModule` is appended to the chart's
# modules list so its writes to `_meta.containerRefs` merge into the chart eval.
# It sources the redis container's image triple from
# `config.firestreamImages.redis.eval (_: {})` (the default alias points at v8
# as of Phase B — see nix/flake-modules/containers/redis.nix) and projects it
# into one slot named "redis" with `componentPath = [ "image" ]` (the chart's
# top-level `image:` block in values.yaml — Bitnami redis uses a single image
# across master/replica/sentinel/exporter pods).
# `global.security.allowInsecureImages` is flipped to bypass Bitnami's
# NOTES.txt whitelist (which rejects the `firestream-redis` repo name).
{ ... }: {
  perSystem = { pkgs, lib, config, evalChart, ... }:
    let
      chartSrc = ../../../src/charts/firestream/redis;
      optionsPath = chartSrc + "/nix/default.nix";

      # Subcharts vendored from the in-repo Bitnami fork. Redis's Chart.yaml
      # lists `common` as its only dependency; the engine's vendor-subcharts.nix
      # handles the nested copy.
      subcharts = [
        { name = "common"; }
      ];

      redisImg =
        let
          imgEval = config.firestreamImages.redis.eval (_: {});
          imgCfg = imgEval.config.redis.image;
        in {
          registry = imgCfg.registry;
          repository = imgCfg.repository;
          tag = imgEval.imageTag;
        };

      imageInjectionModule = { ... }: {
        config.redis._meta.containerRefs.redis = {
          inherit (redisImg) registry repository tag;
          componentPath = [ "image" ];
        };
        config.redis.global.security.allowInsecureImages = true;
      };

      c = evalChart {
        name = "redis";
        inherit chartSrc subcharts;
        modules = [ optionsPath imageInjectionModule ];
      };
    in
    {
      packages.redis-chart = c.chartBundle;

      # Registry: full evaluated chart result (used by aggregate.nix / flake.lib).
      firestreamCharts.redis = c;

      # Registry: consumer override API exposed via flake.lib.<sys>.charts.redis.
      firestreamChartImages.redis = {
        chartBundle = c.chartBundle;
        render = c.render;
        eval = userMod: evalChart {
          name = "redis";
          inherit chartSrc subcharts;
          # Preserve the image injection in user re-evals so consumer overrides
          # still receive the registry-derived image triple by default.
          modules = [ optionsPath imageInjectionModule userMod ];
        };
        options = c.options;
      };
    };
}
