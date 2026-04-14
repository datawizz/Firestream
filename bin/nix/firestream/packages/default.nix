# Firestream Packages Index
# Copyright Firestream. MIT License.
#
# This module provides custom packages built from source within the Firestream
# repository. These packages are used as dependencies in the container module system.
#
# mkRustPackage is provided by the Rust module (Fenix + Crane) for incremental
# builds with dependency caching.
#
# Each package is wrapped with fleet manifest support attributes:
# - metadata: derivation containing SBOM and metadata files
# - isFirestreamArtifact: true for Rust CLI tools (distinguishes from containers)

{ pkgs, lib, mkRustPackage }:

let
  # Raw Rust packages without fleet manifest wrappers
  # These are the actual derivations built from source
  rawPackages = {
    firestream = import ./firestream.nix { inherit pkgs lib mkRustPackage; };
    wait-for-port = import ./wait-for-port.nix { inherit pkgs lib mkRustPackage; };
    firestream-vib = import ./firestream-vib.nix { inherit pkgs lib mkRustPackage; };
  };

  # Import metadata library for generating Rust package metadata
  # Uses the raw firestream-vib package (unwrapped) to generate metadata
  metadataLib = import ../lib/metadata.nix {
    inherit pkgs lib;
    firestreamVibPkg = rawPackages.firestream-vib;
  };

  # Helper function to wrap a Rust package with fleet manifest attributes
  # For CLI tools, we generate metadata based on the built derivation
  wrapRustPackage = { name, package, version ? "0.1.0", description ? "" }:
    let
      # Generate metadata for Rust binary packages
      # This produces SBOM files for the Rust package and its Nix closure
      metadata = metadataLib.mkContainerMetadata {
        inherit name version;
        mainDrv = package;
        # CLI tools don't have ports, users, or working directories
        exposedPorts = [];
        user = "0:0";  # CLI tools typically run as the invoking user
        workdir = "/";
      };
    in
    package // {
      # Expose metadata derivation for fleet SBOM aggregation
      inherit metadata;

      # Expose package list for fleet-level source introspection
      packageList = [ package ];

      # Tag for dynamic filtering (distinguishes from containers)
      isFirestreamArtifact = true;

      # Package metadata
      meta = (package.meta or {}) // {
        inherit name version description;
      };
    };

in {
  # firestream: CLI/TUI for managing data infrastructure services
  # Usage: firestream [COMMAND] or launch the TUI
  firestream = wrapRustPackage {
    name = "firestream-tui";
    package = rawPackages.firestream;
    version = "0.1.0";
    description = "CLI/TUI tool for managing data infrastructure services";
  };

  # wait-for-port: Rust-based port availability checker
  # Usage: wait-for-port PORT [--host HOST] [--state inuse|free] [--timeout SECS]
  wait-for-port = wrapRustPackage {
    name = "wait-for-port";
    package = rawPackages.wait-for-port;
    version = "0.1.0";
    description = "Wait for a TCP port to reach a desired state (inuse or free)";
  };

  # firestream-vib: Container verification harness and metadata generator
  # Usage: firestream-vib generate-metadata --closure-graph FILE --config FILE --output DIR
  firestream-vib = wrapRustPackage {
    name = "firestream-vib";
    package = rawPackages.firestream-vib;
    version = "0.1.0";
    description = "Container verification harness and metadata generator";
  };
}
