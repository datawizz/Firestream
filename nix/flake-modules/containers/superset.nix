# Superset container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the superset container (versions 4 and 5) through the options-driven
# evalContainer entrypoint and contributes:
#   - packages.superset / packages.superset-4 / packages.superset-5
#       (docker images; stub off-Linux). `superset` aliases the v5 build.
#   - firestreamContainers.superset            (v5 module result, for aggregate)
#   - firestreamImages.superset                (v5 consumer override API, flake.lib)
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, firestreamLib, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      options4Path = ../../../src/containers/firestream/superset/4/options.nix;
      options5Path = ../../../src/containers/firestream/superset/5/options.nix;
      workspace4Path = ../../../src/containers/firestream/superset/4;
      workspace5Path = ../../../src/containers/firestream/superset/5;

      # Superset module needs waitForPortPkg, forwarded via extraFactoryArgs.
      mkSuperset = workspacePath: optionsPath: evalContainer {
        name = "superset";
        runtimeType = "python-workspace";
        inherit workspacePath;
        modules = [ optionsPath ];
        extraFactoryArgs = { waitForPortPkg = firestreamLib.waitForPortPkg; };
      };

      c4 = mkSuperset workspace4Path options4Path;
      c5 = mkSuperset workspace5Path options5Path;

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.superset = if isLinux then c5.dockerImage else unavailable "superset";
      packages.superset-4 = if isLinux then c4.dockerImage else unavailable "superset-4";
      packages.superset-5 = if isLinux then c5.dockerImage else unavailable "superset-5";

      # Registry key `superset` -> the v5 result (matches manifest membership).
      firestreamContainers.superset = lib.optionalAttrs isLinux c5;

      firestreamImages.superset = {
        dockerImage = c5.dockerImage;
        eval = userMod: evalContainer {
          name = "superset";
          runtimeType = "python-workspace";
          workspacePath = workspace5Path;
          modules = [ options5Path userMod ];
          extraFactoryArgs = { waitForPortPkg = firestreamLib.waitForPortPkg; };
        };
        options = c5.options;
      };
    };
}
