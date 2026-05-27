# Rust / core packages flake-module
# Copyright Firestream. MIT License.
#
# Ports the legacy flake's non-container packages 1:1:
#   container, default, vib-tools, firestream, wait-for-port, firestream-vib
{ inputs, ... }: {
  perSystem = { pkgs, system, firestreamLib, ... }:
    let
      shellEnv = import ./lib/shell-env.nix {
        inherit pkgs system;
        inherit (inputs) fenix;
      };
    in
    {
      # Container environment derivation (dev profile + packages).
      packages.container = shellEnv.container;

      # Default package: the firestream Rust CLI/TUI built via nix/package.nix.
      packages.default = pkgs.callPackage ../package.nix { };

      # VIB (Validation, Inspection, Build) Tools bundle.
      packages.vib-tools = pkgs.buildEnv {
        name = "firestream-vib-tools";
        paths = [
          pkgs.goss    # Server validation and testing
          pkgs.trivy   # Container vulnerability scanner
          pkgs.grype   # Vulnerability scanner for containers and filesystems
          pkgs.docker  # Docker CLI for container management
        ];
      };

      # Rust packages.
      packages.firestream = pkgs.callPackage ../package.nix { };
      packages.wait-for-port = firestreamLib.packages.wait-for-port;
      packages.firestream-vib = firestreamLib.packages.firestream-vib;
    };
}
