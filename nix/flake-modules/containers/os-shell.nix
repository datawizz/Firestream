# os-shell container flake-module
# Copyright Firestream. MIT License.
#
# Wires the os-shell sidecar container through the options-driven evalContainer
# entrypoint and contributes:
#   - packages.os-shell                  (docker image; stub off-Linux)
#   - firestreamContainers.os-shell      (full module result, for aggregate)
#   - firestreamImages.os-shell          (consumer override API, flake.lib)
#
# runtimeType "system": this is a small POSIX userspace image (bash, coreutils,
# busybox, netcat, curl, gnused) — no JVM, no Python workspace. Single-version
# (`firestream-os-shell:1`); not multi-tagged.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/os-shell/options.nix;
      modulePath = ../../../src/containers/firestream/os-shell/module.nix;

      c = evalContainer {
        name = "os-shell";
        runtimeType = "system";
        inherit modulePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.os-shell = if isLinux then c.dockerImage else unavailable "os-shell";

      firestreamContainers.os-shell = lib.optionalAttrs isLinux c;

      firestreamImages.os-shell = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "os-shell";
          runtimeType = "system";
          inherit modulePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
