# SeaweedFS container flake-module
# Copyright Firestream. MIT License.
#
# Wires the seaweedfs container (single-version object store) through the
# options-driven evalContainer entrypoint and contributes:
#   - packages.seaweedfs            (docker image; stub off-Linux)
#   - firestreamContainers.seaweedfs
#   - firestreamImages.seaweedfs    (image triple for chart injection; Phase C
#                                     reads config.firestreamImages.seaweedfs.eval)
#
# runtimeType "system". SeaweedFS is NOT a Bitnami chart, so the container
# carries no Bitnami-compat scripts (see src/containers/firestream/seaweedfs).
#
# The `weed` binary (bin/nix/firestream/packages/seaweedfs-weed.nix, exposed via
# the Firestream packages index) is threaded to the module through
# extraFactoryArgs, mirroring superset's waitForPortPkg injection.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, firestreamLib, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/seaweedfs/options.nix;
      modulePath = ../../../src/containers/firestream/seaweedfs/module.nix;

      # The `weed` package, threaded to module.nix's `seaweedfsWeedPkg` arg.
      factoryArgs = { seaweedfsWeedPkg = firestreamLib.packages.seaweedfs-weed; };

      c = evalContainer {
        name = "seaweedfs";
        runtimeType = "system";
        inherit modulePath;
        modules = [ optionsPath ];
        extraFactoryArgs = factoryArgs;
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.seaweedfs = if isLinux then c.dockerImage else unavailable "seaweedfs";

      firestreamContainers.seaweedfs = lib.optionalAttrs isLinux c;

      firestreamImages.seaweedfs = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "seaweedfs";
          runtimeType = "system";
          inherit modulePath;
          modules = [ optionsPath userMod ];
          extraFactoryArgs = factoryArgs;
        };
        options = c.options;
      };
    };
}
