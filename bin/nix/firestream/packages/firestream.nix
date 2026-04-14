# firestream Nix package
# Copyright Firestream. MIT License.
#
# Builds the Rust firestream CLI/TUI binary for managing data infrastructure services.
#
# Requires mkRustPackage (Crane-based) for workspace dependency resolution.

{ pkgs, lib, mkRustPackage }:

let
  workspaceSrc = ../../../../.;
in
mkRustPackage {
  name = "firestream";
  src = workspaceSrc;
  cargoExtraArgs = "-p firestream";

  sourceFilter = path: _type:
    (builtins.match ".*/_python_fastapi/.*" path) != null;

  meta = {
    description = "CLI/TUI tool for managing data infrastructure services";
    mainProgram = "firestream";
  };
}
