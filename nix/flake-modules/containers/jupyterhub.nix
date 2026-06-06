# JupyterHub container flake-module (Phase 4)
# Copyright Firestream. MIT License.
#
# Wires the jupyterhub container through the options-driven evalContainer
# entrypoint and contributes:
#   - packages.jupyterhub / packages.jupyterhub-5  (docker images; stub off-Linux)
#   - firestreamContainers.jupyterhub              (full module result, for aggregate)
#   - firestreamImages.jupyterhub                  (consumer override API, flake.lib)
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/jupyterhub/options.nix;
      workspacePath = ../../../src/containers/firestream/jupyterhub;

      c = evalContainer {
        name = "jupyterhub";
        runtimeType = "python-workspace";
        inherit workspacePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.jupyterhub = if isLinux then c.dockerImage else unavailable "jupyterhub";
      packages.jupyterhub-5 = if isLinux then c.dockerImage else unavailable "jupyterhub-5";

      firestreamContainers.jupyterhub = lib.optionalAttrs isLinux c;

      firestreamImages.jupyterhub = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "jupyterhub";
          runtimeType = "python-workspace";
          inherit workspacePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
