# metadata.nix - Container metadata and SBOM generation
# Copyright Firestream. MIT License.
#
# This module provides functions for generating container metadata at Nix build time:
# - metadata.json: Core container and build information
# - sbom-cyclonedx.json: CycloneDX 1.5 compliant SBOM
# - sbom-spdx.json: SPDX 2.3 compliant SBOM
# - closure.json: Nix closure dependency tree
#
# Uses the Rust-based firestream-vib tool for efficient metadata generation
# without requiring Python as a build dependency.
#
# Usage:
#   let
#     metadata = mkContainerMetadata {
#       name = "airflow";
#       version = "2.8.0";
#       mainDrv = airflowPackage;
#       exposedPorts = [ 8080 ];
#       user = "1001:1001";
#       workdir = "/opt/firestream/airflow";
#       nixpkgsRevision = "abc123...";
#       flakeUri = "github:Cogent-Creation-Co/Firestream";
#       flakeRevision = "def456...";
#     };
#   in metadata  # Derivation containing all metadata files

{ pkgs, lib, firestreamVibPkg }:

let
  # Define mkContainerMetadata in the let block
  mkContainerMetadata = {
    # Required parameters
    name,                       # Container name (e.g., "airflow")
    version ? "1.0.0",          # Container version
    mainDrv,                    # Main derivation to analyze closure of

    # Container configuration
    exposedPorts ? [],          # List of exposed port numbers
    user ? "1001:1001",         # Container user (uid:gid format)
    workdir ? "/opt/firestream/${name}",  # Working directory

    # Build provenance
    nixpkgsRevision ? "",       # Nixpkgs commit hash
    flakeUri ? "",              # Source flake URI
    flakeRevision ? "",         # Source flake revision/commit
  }:
  let
    # Configuration file for firestream-vib
    configJson = pkgs.writeText "metadata-config.json" (builtins.toJSON {
      container_name = name;
      container_version = version;
      main_store_path = builtins.toString mainDrv;
      exposed_ports = exposedPorts;
      user = user;
      workdir = workdir;
      nixpkgs_revision = nixpkgsRevision;
      flake_uri = flakeUri;
      flake_revision = flakeRevision;
    });

  in pkgs.stdenv.mkDerivation {
    pname = "${name}-metadata";
    inherit version;

    # No source needed - we generate everything
    dontUnpack = true;

    # Use exportReferencesGraph to get the Nix closure at build time
    # This creates a file called "closure-graph" with all store paths and references
    # Format: store_path\nref_count\nref1\nref2\n...\nstore_path\n...
    exportReferencesGraph = [ "closure-graph" mainDrv ];

    nativeBuildInputs = [ firestreamVibPkg ];

    buildPhase = ''
      runHook preBuild

      echo "Generating container metadata for ${name} v${version}..."
      echo "Using firestream-vib for metadata generation"

      # Run the Rust metadata generator
      firestream-vib generate-metadata \
        --closure-graph closure-graph \
        --config ${configJson} \
        --output opt/firestream

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      # Create output directory structure
      mkdir -p $out/opt/firestream

      # Copy generated files to output
      cp opt/firestream/metadata.json $out/opt/firestream/
      cp opt/firestream/sbom-cyclonedx.json $out/opt/firestream/
      cp opt/firestream/sbom-spdx.json $out/opt/firestream/
      cp opt/firestream/closure.json $out/opt/firestream/

      # Verify all base files were created
      for f in metadata.json sbom-cyclonedx.json sbom-spdx.json closure.json; do
        if [ ! -f "$out/opt/firestream/$f" ]; then
          echo "ERROR: Failed to generate $f"
          exit 1
        fi
        echo "Generated: $out/opt/firestream/$f"
      done

      runHook postInstall
    '';

    meta = with lib; {
      description = "Container metadata and SBOMs for ${name}";
      license = licenses.mit;
      maintainers = [ ];
    };
  };

  # Helper function to get metadata from an existing container module
  # Useful for integrating with mkContainerModule
  mkMetadataForContainer = containerModule: {
    name,
    version ? containerModule.meta.version or "1.0.0",
    nixpkgsRevision ? "",
    flakeUri ? "",
    flakeRevision ? "",
  }:
    let
      inherit (containerModule) config;
    in
    mkContainerMetadata {
      inherit name version nixpkgsRevision flakeUri flakeRevision;
      mainDrv = containerModule.runtimeEnv;
      exposedPorts = config.exposedPorts or [];
      user = "${toString (config.user.uid or 1001)}:${toString (config.user.gid or 1001)}";
      workdir = config.paths.base or "/opt/firestream/${name}";
    };

in
{
  meta = {
    name = "libmetadata";
    description = "Container metadata and SBOM generation at Nix build time";
    version = "2.0.0";  # Bumped version for Rust implementation
  };

  # Export the factory function
  inherit mkContainerMetadata;

  # Export the helper function
  inherit mkMetadataForContainer;
}
