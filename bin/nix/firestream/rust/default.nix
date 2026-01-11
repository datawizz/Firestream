# Firestream Rust Module System
# Copyright Firestream. Apache-2.0 License.
#
# Orchestrates Fenix toolchain and Crane build system for Rust packages.
# Provides deterministic, incremental Rust builds with dependency caching.

{ pkgs, fenix, crane, system }:

let
  lib = pkgs.lib;

  # Import toolchain configuration
  toolchainModule = import ./toolchain.nix { inherit pkgs fenix system; };

  # Import Darwin configuration
  darwinModule = import ./darwin.nix { inherit pkgs lib; };

  # Import package builder
  rustPackageModule = import ./mkRustPackage.nix {
    inherit pkgs lib crane;
    inherit toolchainModule;
  };

in {
  # Toolchain access
  toolchain = toolchainModule.toolchain;
  toolchainModule = toolchainModule;

  # Package builder
  mkRustPackage = rustPackageModule.mkRustPackage;

  # Crane lib for advanced usage
  craneLib = rustPackageModule.craneLib;

  # Darwin utilities
  darwin = darwinModule;

  # Development shell with Rust toolchain
  mkDevShell = { extraPackages ? [], extraShellHook ? "" }:
    pkgs.mkShell {
      buildInputs = [
        toolchainModule.toolchain
        pkgs.pkg-config
        pkgs.openssl
        pkgs.openssl.dev
        pkgs.zlib
        pkgs.zstd
        pkgs.bzip2
        pkgs.llvmPackages.clang
        pkgs.llvmPackages.libclang.lib
      ] ++ extraPackages
        ++ darwinModule.frameworks;

      nativeBuildInputs = [ pkgs.pkg-config ];

      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      RUST_SRC_PATH = toolchainModule.rustSrcPath;

      shellHook = ''
        export CARGO_HOME="$HOME/.cargo"
        export PATH="$CARGO_HOME/bin:$PATH"
        ${extraShellHook}
      '';
    };

  # Meta information
  meta = {
    name = "firestream-rust-module";
    version = "1.0.0";
    description = "Fenix + Crane based Rust build system for Firestream";
  };
}
