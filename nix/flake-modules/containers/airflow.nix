# Airflow container flake-module (Phase 3)
# Copyright Firestream. MIT License.
#
# Wires the airflow container through the options-driven evalContainer entrypoint
# and contributes:
#   - packages.airflow / packages.airflow-3   (docker images; n/a stub off-Linux)
#   - firestreamContainers.airflow            (full module result, for aggregate)
#   - firestreamImages.airflow                (consumer override API, for flake.lib)
#
# The other 7 containers (odoo*, jupyterhub, superset, kafka, spark) are wired in
# Phase 4 and are intentionally absent here.
{ ... }: {
  perSystem = { pkgs, lib, system, evalContainer, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      optionsPath = ../../../src/containers/firestream/airflow/options.nix;
      workspacePath = ../../../src/containers/firestream/airflow;

      c = evalContainer {
        name = "airflow";
        runtimeType = "python-workspace";
        inherit workspacePath;
        modules = [ optionsPath ];
      };

      unavailable = n: pkgs.runCommand "${n}-not-available" { } ''
        echo "Docker images only available on Linux systems" > $out
      '';
    in
    {
      packages.airflow = if isLinux then c.dockerImage else unavailable "airflow";
      packages.airflow-3 = if isLinux then c.dockerImage else unavailable "airflow-3";

      # Registry: full evaluated module result (used by aggregate.nix manifest/sbom).
      # Only populated on Linux, matching the legacy isLinux-gated container set.
      firestreamContainers.airflow = lib.optionalAttrs isLinux c;

      # Registry: consumer override API exposed via flake.lib.<sys>.images.airflow.
      firestreamImages.airflow = {
        dockerImage = c.dockerImage;
        eval = userMod: evalContainer {
          name = "airflow";
          runtimeType = "python-workspace";
          inherit workspacePath;
          modules = [ optionsPath userMod ];
        };
        options = c.options;
      };
    };
}
