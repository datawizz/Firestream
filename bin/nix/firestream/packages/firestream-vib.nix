# firestream-vib Nix package
# Copyright Firestream. MIT License.
#
# Builds the Rust firestream-vib binary for container verification and metadata generation.
# This tool provides:
# - Container verification harness (Goss, Trivy, Grype)
# - SBOM generation (CycloneDX 1.5, SPDX 2.3)
# - Nix closure metadata generation for container builds
#
# Requires mkRustPackage (Crane-based) for workspace dependency resolution.

{ pkgs, lib, mkRustPackage }:

let
  # The workspace root
  workspaceSrc = ../../../../.;
in
mkRustPackage {
  name = "firestream-vib";
  src = workspaceSrc;
  cargoExtraArgs = "-p firestream-vib";

  # Additional build inputs for OpenSSL and other dependencies
  buildInputs = with pkgs; [
    openssl
    openssl.dev
  ];

  meta = {
    description = "Container verification harness and metadata generator for Nix-built containers";
    mainProgram = "firestream-vib";
  };
}
