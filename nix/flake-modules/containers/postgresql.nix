# PostgreSQL container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the postgresql container (multi-version 16/17) through the options-driven
# evalContainer entrypoint and contributes:
#   - packages.postgresql / packages.postgresql-16 / packages.postgresql-17
#       (docker images; stub off-Linux). `postgresql` aliases the v17 build.
#   - firestreamContainers.postgresql / postgresql-16 / postgresql-17
#   - firestreamImages.postgresql / postgresql-16 / postgresql-17
#
# runtimeType "system": version varies per build, supplied via an inline override
# module. NOT in the fleet manifest membership set.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/postgresql/options.nix;
      modulePath = ../../../src/containers/firestream/postgresql/module.nix;

      mkPostgresql = v: evalContainer {
        name = "postgresql";
        runtimeType = "system";
        inherit modulePath;
        modules = [
          optionsPath
          ({ ... }: { config.postgresql.version = v; })
        ];
      };

      c16 = mkPostgresql "16";
      c17 = mkPostgresql "17";

      mkImage = v: c: {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "postgresql";
          runtimeType = "system";
          inherit modulePath;
          modules = [
            optionsPath
            ({ ... }: { config.postgresql.version = v; })
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
      packages.postgresql = if isLinux then c17.dockerImage else unavailable "postgresql";
      packages.postgresql-16 = if isLinux then c16.dockerImage else unavailable "postgresql-16";
      packages.postgresql-17 = if isLinux then c17.dockerImage else unavailable "postgresql-17";

      firestreamContainers.postgresql = lib.optionalAttrs isLinux c17;
      firestreamContainers.postgresql-16 = lib.optionalAttrs isLinux c16;
      firestreamContainers.postgresql-17 = lib.optionalAttrs isLinux c17;

      firestreamImages.postgresql = mkImage "17" c17;
      firestreamImages.postgresql-16 = mkImage "16" c16;
      firestreamImages.postgresql-17 = mkImage "17" c17;
    };
}
