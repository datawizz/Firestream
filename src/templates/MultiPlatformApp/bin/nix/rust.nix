# rust.nix - Rust development environment for containers
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null, lib ? pkgs.lib }:

let
  # Rust toolchain with required components and cross-compilation targets
  # Priority: fenix (from flake input) > rust-bin overlay > nixpkgs fallback
  rustToolchain = if fenixPackages != null then
    fenixPackages.combine ([
      (fenixPackages.stable.withComponents [
        "cargo"
        "clippy"
        "rust-src"
        "rust-analyzer"
        "rustc"
        "rustfmt"
      ])
      fenixPackages.targets.wasm32-unknown-unknown.stable.rust-std
    ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
      fenixPackages.targets.x86_64-unknown-linux-gnu.stable.rust-std
    ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      fenixPackages.targets.aarch64-apple-ios.stable.rust-std
      fenixPackages.targets.aarch64-apple-ios-sim.stable.rust-std
      fenixPackages.targets.x86_64-apple-ios.stable.rust-std
    ])
  else if pkgs ? rust-bin then
    (pkgs.rust-bin.stable.latest.default.override {
      extensions = [
        "rust-src"
        "rust-analyzer"
        "clippy"
        "rustfmt"
      ];
      targets = [
        "wasm32-unknown-unknown"
        "x86_64-unknown-linux-gnu"
      ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
        "aarch64-apple-ios"
        "aarch64-apple-ios-sim"
        "x86_64-apple-ios"
      ];
    })
  else
    # Fallback for standard nixpkgs (without fenix or rust-bin)
    pkgs.symlinkJoin {
      name = "rust-toolchain";
      paths = with pkgs; [
        rustc
        cargo
        rust-analyzer
        clippy
        rustfmt
      ];
    };

  # Essential packages for Rust development
  buildInputs = with pkgs; [
    # Build essentials
    pkg-config
    openssl
    openssl.dev

    # Common C libraries
    zlib
    zlib.dev
    xz
    xz.dev

    # For linking
    gcc
    llvmPackages.clang  # Needed for lld
    llvmPackages.libclang.lib
    llvmPackages.bintools
    llvmPackages.lld  # LLVM linker (cross-platform)

    # Additional libraries often needed
    libiconv
    libffi
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    mold  # Fast linker (Linux only, broken on darwin)
  ];

  # Cargo tools
  cargoTools = with pkgs; [
    cargo-edit      # Add/remove/upgrade dependencies
    cargo-watch     # Watch for changes and rebuild
    cargo-expand    # Expand macros
    cargo-outdated  # Check for outdated dependencies
  ];

  # Optional: sccache for faster builds
  sccache = pkgs.sccache;

  # Environment variables for Rust
  envVars = ''
    # Rust source path for rust-analyzer
    export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

    # Cargo home
    export CARGO_HOME="$HOME/.cargo"
    export PATH="$CARGO_HOME/bin:$PATH"

    # Optional: Use sccache for faster builds
    # Comment out these lines if you want to use incremental compilation instead
    export RUSTC_WRAPPER="${sccache}/bin/sccache"
    export SCCACHE_CACHE_SIZE="50G"
    export SCCACHE_DIR="$PWD/.sccache"
    export SCCACHE_IDLE_TIMEOUT=0
    export SCCACHE_ERROR_LOG="$PWD/.sccache/error.log"
    export CARGO_TARGET_DIR="$PWD/target"

    # RUSTFLAGS moved to cargo config for better control

    # Alternative: To disable sccache and use incremental compilation:
    # unset RUSTC_WRAPPER
    # export CARGO_INCREMENTAL=1

    # OpenSSL configuration
    export OPENSSL_DIR="${pkgs.openssl.dev}"
    export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
    export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

    # PKG_CONFIG for finding system libraries
    export PKG_CONFIG_PATH="${lib.makeSearchPath "lib/pkgconfig" [
      pkgs.openssl.dev
      pkgs.xz
      pkgs.zlib.dev
    ]}:$PKG_CONFIG_PATH"

    # C library paths for building native dependencies
    export C_INCLUDE_PATH="${lib.makeSearchPath "include" [
      pkgs.openssl.dev
      pkgs.xz.dev
      pkgs.zlib.dev
    ]}:$C_INCLUDE_PATH"

    export LIBRARY_PATH="${lib.makeSearchPath "lib" [
      pkgs.openssl.out
      pkgs.xz
      pkgs.zlib
    ]}:$LIBRARY_PATH"

    # LLVM/Clang for bindgen
    export LIBCLANG_PATH="${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages.libclang.lib ]}"

    # Additional build flags
    export CFLAGS="-I${pkgs.openssl.dev}/include -I${pkgs.zlib.dev}/include"
    export LDFLAGS="-L${pkgs.openssl.out}/lib -L${pkgs.zlib}/lib"

    # Create sccache and target directories if they don't exist
    mkdir -p "$PWD/.sccache" 2>/dev/null || true
    mkdir -p "$PWD/target" 2>/dev/null || true

    # Rust-specific optimizations
    # Note: CARGO_INCREMENTAL must be 0 when using sccache
    export CARGO_INCREMENTAL=0
    export RUST_BACKTRACE=1
  '';

  # Setup script for initial configuration
  setupScript = pkgs.writeScript "setup-rust" ''
    #!${pkgs.bash}/bin/bash
    set -e

    # Create cargo directories
    mkdir -p "$HOME/.cargo/bin"

    # Always recreate cargo config to ensure it's up to date
    cat > "$HOME/.cargo/config.toml" <<EOF
[build]
# Use sccache for compilation caching (target-dir set via CARGO_TARGET_DIR env var)
rustc-wrapper = "${sccache}/bin/sccache"

${if pkgs.stdenv.isLinux then ''
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "split-debuginfo=unpacked"]
'' else "# mold linker not available on darwin, using default linker"}

[profile.dev]
incremental = false  # Required for sccache
debug = 1
split-debuginfo = "unpacked"

# Optimize all dependencies
[profile.dev.package."*"]
opt-level = 3
codegen-units = 16
debug = false

[net]
git-fetch-with-cli = true
EOF
  '';

in
{
  # All packages to install
  packages = [ rustToolchain sccache ] ++ buildInputs ++ cargoTools;

  # Environment variables
  inherit envVars;

  # Shell hook for development
  shellHook = ''
    ${envVars}
  '';

  # Profile script for persistent environment
  profileScript = ''
    ${envVars}
  '';

  # Setup script for initialization
  setupScript = setupScript;

  # No apps for Rust module
  apps = {};
}
