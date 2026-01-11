# Crane-based Rust Package Builder
# Copyright Firestream. Apache-2.0 License.
#
# Provides incremental Rust builds with dependency caching via Crane.
# Uses buildDepsOnly to create a cached dependency layer that is
# reused across package builds.

{ pkgs, lib, crane, toolchainModule }:

let
  # Create crane lib with our Fenix toolchain
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchainModule.toolchain;

  # Import Darwin configuration
  darwinModule = import ./darwin.nix { inherit pkgs lib; };

  # Common build inputs for all Rust packages
  commonBuildInputs = with pkgs; [
    openssl
    openssl.dev
    zlib
    zstd
    bzip2
    xz
    libiconv
  ] ++ darwinModule.frameworks;

  # Native build inputs (build-time only)
  commonNativeBuildInputs = with pkgs; [
    pkg-config
    llvmPackages.clang
    llvmPackages.libclang.lib
  ];

  # Common environment variables for Rust builds
  commonEnv = {
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    OPENSSL_DIR = "${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  };

in {
  # Main package builder function
  mkRustPackage = {
    name,
    src,
    version ? "0.1.0",
    cargoExtraArgs ? "",
    buildInputs ? [],
    nativeBuildInputs ? [],
    env ? {},
    meta ? {},
    ...
  }@args:
    let
      # Filter source to only Rust-relevant files
      cleanedSrc = craneLib.cleanCargoSource src;

      # Common arguments for both deps and final build
      commonArgs = {
        pname = name;
        inherit version;
        src = cleanedSrc;

        buildInputs = commonBuildInputs ++ buildInputs;
        nativeBuildInputs = commonNativeBuildInputs ++ nativeBuildInputs;

        # Merge environment variables
        inherit (commonEnv) LIBCLANG_PATH OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_INCLUDE_DIR;
      } // env;

      # Build only the dependencies (cached layer)
      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
        pname = "${name}-deps";
      });

    in craneLib.buildPackage (commonArgs // {
      inherit cargoArtifacts cargoExtraArgs;

      meta = {
        description = args.description or "Firestream ${name}";
        homepage = "https://github.com/Cogent-Creation-Co/Firestream";
        license = lib.licenses.asl20;
        maintainers = [ "Firestream Team" ];
        mainProgram = args.mainProgram or name;
      } // meta;
    });

  # Export crane lib for advanced usage
  inherit craneLib;

  # Export toolchain module
  inherit toolchainModule;

  # Export common build inputs for extension
  inherit commonBuildInputs commonNativeBuildInputs commonEnv;
}
