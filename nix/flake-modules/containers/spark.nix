# Spark container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the apache spark container through the options-driven evalContainer
# entrypoint and contributes:
#   - packages.spark / packages.spark-4  (docker images; stub off-Linux)
#   - firestreamContainers.spark         (full module result, for aggregate)
#   - firestreamImages.spark             (consumer override API, flake.lib)
#
# runtimeType "java": `jdk` and `python` remain module defaults (NOT passed here).
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/spark/options.nix;
      modulePath = ../../../src/containers/firestream/spark/module.nix;

      c = evalContainer {
        name = "spark";
        runtimeType = "java";
        inherit modulePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.spark = if isLinux then c.dockerImage else unavailable "spark";
      packages.spark-4 = if isLinux then c.dockerImage else unavailable "spark-4";

      firestreamContainers.spark = lib.optionalAttrs isLinux c;

      firestreamImages.spark = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "spark";
          runtimeType = "java";
          inherit modulePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
