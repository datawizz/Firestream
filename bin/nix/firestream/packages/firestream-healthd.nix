# firestream-healthd Nix package
# Copyright Firestream. MIT License.
#
# Builds the Rust firestream-healthd binary for in-container introspection.
# This tool exposes a tiny blocking HTTP server with these endpoints:
#   GET /healthz   — liveness ("ok")
#   GET /readyz    — readiness (per-image configurable; cmd/tcp/http)
#   GET /metadata  — streams /opt/firestream/metadata.json
#   GET /sbom      — streams /opt/firestream/sbom-cyclonedx.json (or ?format=spdx)
#   GET /closure   — streams /opt/firestream/closure.json
#
# Requires mkRustPackage (Crane-based) for workspace dependency resolution.

{ pkgs, lib, mkRustPackage }:

let
  # The workspace root
  workspaceSrc = ../../../../.;
in
mkRustPackage {
  name = "firestream-healthd";
  src = workspaceSrc;
  cargoExtraArgs = "-p firestream-healthd";

  meta = {
    description = "Tiny in-image health/SBOM HTTP server for Firestream containers";
    mainProgram = "firestream-healthd";
  };
}
