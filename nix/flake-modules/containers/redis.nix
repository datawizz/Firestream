# Redis container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the redis container (multi-version 7/8) through the options-driven
# evalContainer entrypoint and contributes:
#   - packages.redis / packages.redis-7 / packages.redis-8
#       (docker images; stub off-Linux). `redis` aliases the v7 build.
#   - firestreamContainers.redis / redis-7 / redis-8
#   - firestreamImages.redis / redis-7 / redis-8
#
# runtimeType "system": version varies per build, supplied via an inline override
# module. NOT in the fleet manifest membership set. The legacy module selects the
# same `pkgs.redis` for both versions; parity is preserved via version-in-tag.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/redis/options.nix;
      modulePath = ../../../src/containers/firestream/redis/module.nix;

      mkRedis = v: evalContainer {
        name = "redis";
        runtimeType = "system";
        inherit modulePath;
        modules = [
          optionsPath
          ({ ... }: { config.redis.version = v; })
        ];
      };

      c7 = mkRedis "7";
      c8 = mkRedis "8";

      mkImage = v: c: {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "redis";
          runtimeType = "system";
          inherit modulePath;
          modules = [
            optionsPath
            ({ ... }: { config.redis.version = v; })
            userMod
          ];
        };
        options = c.options;
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.redis = if isLinux then c7.dockerImage else unavailable "redis";
      packages.redis-7 = if isLinux then c7.dockerImage else unavailable "redis-7";
      packages.redis-8 = if isLinux then c8.dockerImage else unavailable "redis-8";

      firestreamContainers.redis = lib.optionalAttrs isLinux c7;
      firestreamContainers.redis-7 = lib.optionalAttrs isLinux c7;
      firestreamContainers.redis-8 = lib.optionalAttrs isLinux c8;

      firestreamImages.redis = mkImage "7" c7;
      firestreamImages.redis-7 = mkImage "7" c7;
      firestreamImages.redis-8 = mkImage "8" c8;
    };
}
