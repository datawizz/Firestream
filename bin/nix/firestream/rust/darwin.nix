# Darwin-specific Rust Configuration
# Copyright Firestream. Apache-2.0 License.
#
# Provides macOS SDK configuration for Rust builds on Darwin.
#
# In nixpkgs 25.11+, the Darwin SDK pattern changed:
# - Frameworks are bundled in the SDK (apple-sdk)
# - The stdenv provides a default SDK automatically
# - libiconv, libresolv, libsbuf are propagated automatically
# - MACOSX_DEPLOYMENT_TARGET is set by stdenv (currently 14.0)
#
# See: https://github.com/NixOS/nixpkgs/issues/354146

{ pkgs, lib }:

let
  isDarwin = pkgs.stdenv.isDarwin;

  # In nixpkgs 25.11+, frameworks come from the SDK automatically
  # The stdenv provides the SDK and sets DEVELOPER_DIR/SDKROOT
  darwinBuildInputs = if isDarwin then [
    # libiconv is propagated automatically by the SDK in 25.11+
    # but we include it explicitly for compatibility
    pkgs.libiconv
  ] else [];

in {
  # Frameworks/build inputs for Darwin
  # Minimal now since the SDK provides frameworks automatically
  frameworks = darwinBuildInputs;

  # Helper to add Darwin deps to existing buildInputs
  withDarwinDeps = buildInputs:
    buildInputs ++ darwinBuildInputs;

  # Check if running on Darwin
  inherit isDarwin;
}
