# Aggregate flake-module
# Copyright Firestream. MIT License.
#
# Reads the container registry (config.firestreamContainers) DYNAMICALLY so that
# as Phase 4 wires more containers their artifacts automatically flow into the
# fleet manifest, legacyPackages.containers, and legacyPackages.sbom.
#
# legacyPackages is used (not packages) for the nested `containers` / `sbom`
# attrsets because flake-parts requires `packages` to be a flat attrset of
# derivations.
{ inputs, ... }: {
  perSystem = { config, pkgs, lib, firestreamLib, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;

      # Whatever containers are wired so far (Phase 3: just airflow).
      reg = config.firestreamContainers;

      # Manifest membership filter: the FINAL target set. Only members actually
      # present in `reg` are included, so this auto-expands across phases.
      wanted = [ "airflow" "odoo" "odoo-15" "odoo-16" "odoo-17" "jupyterhub" "superset" "kafka" "spark" ];
      members = lib.filterAttrs (n: _: builtins.elem n wanted) reg;

      # Container modules carry metadata via `.module`; rust CLI tools come from
      # the metadata-wrapped firestreamLib.packages.* derivations.
      artifacts = (lib.mapAttrs (_: c: c.module) members) // {
        firestream-tui = firestreamLib.packages.firestream;
        wait-for-port = firestreamLib.packages.wait-for-port;
        firestream-vib = firestreamLib.packages.firestream-vib;
      };
    in
    {
      # ====================================================================
      # FLEET MANIFEST
      # Usage: nix build .#manifest
      # ====================================================================
      packages.manifest =
        if isLinux then
          firestreamLib.manifest.mkFleetManifest {
            artifacts = firestreamLib.manifest.collectArtifacts artifacts;
            fleetName = "firestream";
            version = inputs.self.rev or inputs.self.dirtyRev or "dev";
            archiveSources = true;
          }
        else
          pkgs.runCommand "manifest-not-available" { } ''
            echo "Docker images only available on Linux systems" > $out
          '';

      # ====================================================================
      # Container images namespace (moved out of packages -> legacyPackages)
      # Usage: nix build .#legacyPackages.<sys>.containers.airflow
      # ====================================================================
      legacyPackages.containers =
        lib.optionalAttrs isLinux (lib.mapAttrs (_: c: c.dockerImage) reg);

      # ====================================================================
      # Individual SBOM access (moved out of packages -> legacyPackages)
      # Usage: nix build .#legacyPackages.<sys>.sbom.airflow
      # ====================================================================
      legacyPackages.sbom =
        lib.optionalAttrs isLinux (
          (lib.mapAttrs (_: c: c.module.metadata) members) // {
            firestream-tui = firestreamLib.packages.firestream.metadata;
            wait-for-port = firestreamLib.packages.wait-for-port.metadata;
            firestream-vib = firestreamLib.packages.firestream-vib.metadata;
          }
        );
    };
}
