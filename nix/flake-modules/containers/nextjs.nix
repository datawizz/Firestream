# Next.js container flake-module
# Copyright Firestream. MIT License.
#
# Wires the net-new nextjs container through the options-driven evalContainer
# entrypoint (runtimeType = "system" + modulePath, like kafka/spark) and
# contributes:
#   - packages.nextjs              (docker image; stub off-Linux)
#   - firestreamContainers.nextjs  (full module result, for aggregate)
#   - firestreamImages.nextjs      (consumer override API + compose, flake.lib)
#
# The module.nix factory builds the Next.js app with mkNodePackage and wraps it
# with mkNodeContainerModule; both are exposed on `firestream` (firestreamLib),
# which evalContainer passes to the module for the "system" runtimeType.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/nextjs/options.nix;
      modulePath = ../../../src/containers/firestream/nextjs/module.nix;

      c = evalContainer {
        name = "nextjs";
        runtimeType = "system";
        inherit modulePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.nextjs = if isLinux then c.dockerImage else unavailable "nextjs";

      firestreamContainers.nextjs = lib.optionalAttrs isLinux c;

      firestreamImages.nextjs = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "nextjs";
          runtimeType = "system";
          inherit modulePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
