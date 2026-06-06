# Kafka container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the kafka container through the options-driven evalContainer entrypoint
# and contributes:
#   - packages.kafka / packages.kafka-4  (docker images; stub off-Linux)
#   - firestreamContainers.kafka         (full module result, for aggregate)
#   - firestreamImages.kafka             (consumer override API, flake.lib)
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/kafka/options.nix;
      modulePath = ../../../src/containers/firestream/kafka/module.nix;

      c = evalContainer {
        name = "kafka";
        runtimeType = "java";
        inherit modulePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.kafka = if isLinux then c.dockerImage else unavailable "kafka";
      packages.kafka-4 = if isLinux then c.dockerImage else unavailable "kafka-4";

      firestreamContainers.kafka = lib.optionalAttrs isLinux c;

      firestreamImages.kafka = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "kafka";
          runtimeType = "java";
          inherit modulePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
