# wait-for-port Nix package
# Copyright Firestream. Apache-2.0 License.
#
# Builds the Rust wait-for-port binary for container use.
# This tool waits for a TCP port to reach a desired state (inuse or free).

{ pkgs, lib }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "wait-for-port";
  version = "0.1.0";

  # Source from workspace
  src = ../../../../src/lib/rust/tools/wait-for-port;

  # Use workspace Cargo.lock for dependency resolution
  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  # Build only wait-for-port from the workspace
  cargoBuildFlags = [ "-p" "wait-for-port" ];
  cargoTestFlags = [ "-p" "wait-for-port" ];

  # Native build inputs for compilation
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  meta = with lib; {
    description = "Wait for a TCP port to reach a desired state (inuse or free)";
    homepage = "https://github.com/Cogent-Creation-Co/Firestream";
    license = licenses.mit;
    maintainers = [ "Firestream Team" ];
    mainProgram = "wait-for-port";
  };
}
