# Rust Toolchain via Fenix
# Copyright Firestream. Apache-2.0 License.
#
# Provides a deterministic Rust toolchain using Fenix.
# Edition 2024 is fully supported in nixpkgs 25.11 (Rust 1.85+).

{ pkgs, fenix, system }:

let
  stable = fenix.packages.${system}.stable;

  # Combined stable toolchain with all development tools
  rustToolchain = fenix.packages.${system}.combine [
    stable.rustc
    stable.cargo
    stable.clippy
    stable.rustfmt
    stable.rust-src
    stable.rust-analyzer
  ];

in {
  # Primary toolchain
  toolchain = rustToolchain;

  # Convenience aliases
  stable = rustToolchain;
  default = rustToolchain;

  # Individual components for granular access
  components = {
    rustc = stable.rustc;
    cargo = stable.cargo;
    clippy = stable.clippy;
    rustfmt = stable.rustfmt;
    rust-src = stable.rust-src;
    rust-analyzer = stable.rust-analyzer;
  };

  # Rust source path for IDE integration
  rustSrcPath = "${rustToolchain}/lib/rustlib/src/rust/library";
}
