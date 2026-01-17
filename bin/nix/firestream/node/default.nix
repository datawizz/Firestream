# bin/nix/firestream/node/default.nix
# Node.js Package Building Module
#
# Provides utilities for building Node.js packages with Nix, with focus on:
# - pnpm as primary package manager
# - TypeScript support with auto-detection
# - buildNpmPackage wrapper for consistent builds
#
# Usage:
#   let
#     nodeModule = import ./node { inherit pkgs lib; };
#     myApp = nodeModule.mkNodePackage { ... };
#   in myApp

{ pkgs, lib }:

let
  # Import the package builder
  mkNodePackageModule = import ./mkNodePackage.nix { inherit pkgs lib; };

in {
  # Main package builder
  inherit (mkNodePackageModule) mkNodePackage;

  # Export utilities for advanced usage
  inherit (mkNodePackageModule) detectPackageManager hasTypeScript;

  # Default Node.js version
  nodejs = pkgs.nodejs_22;

  # Module metadata
  meta = {
    name = "firestream-node";
    description = "Node.js package building utilities for Firestream";
    nodeVersion = pkgs.nodejs_22.version;
  };
}
