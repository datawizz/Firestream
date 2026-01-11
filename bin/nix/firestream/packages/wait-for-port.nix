# wait-for-port Nix package
# Copyright Firestream. Apache-2.0 License.
#
# Builds the Rust wait-for-port binary for container use.
# This tool waits for a TCP port to reach a desired state (inuse or free).
#
# Requires mkRustPackage (Crane-based) for workspace dependency resolution.

{ pkgs, lib, mkRustPackage }:

let
  # The workspace root
  workspaceSrc = ../../../../.;
in
mkRustPackage {
  name = "wait-for-port";
  src = workspaceSrc;
  cargoExtraArgs = "-p wait-for-port";

  meta = {
    description = "Wait for a TCP port to reach a desired state (inuse or free)";
    mainProgram = "wait-for-port";
  };
}
