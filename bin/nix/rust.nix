# rust.nix - Rust development environment for containers
{ pkgs, lib ? pkgs.lib }:

let
  # Use rust-bin overlay if available, otherwise fall back to rustPlatform
  rustChannel = if pkgs ? rust-bin then
    pkgs.rust-bin.stable.latest.default
  else
    pkgs.rustPlatform.rust.stable;

  # Rust toolchain with required components
  rustToolchain = if pkgs ? rust-bin then
    rustChannel.override {
      extensions = [
        "rust-src"
        "rust-analyzer"
        "clippy"
        "rustfmt"
      ];
      targets = [
        "wasm32-unknown-unknown"
        "x86_64-unknown-linux-gnu"
      ];
    }
  else
    # Fallback for standard nixpkgs (without rust-bin overlay)
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
    llvmPackages.libclang.lib
    llvmPackages.bintools
    
    # Additional libraries often needed
    libiconv
    libffi
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

in {
  # All packages to install
  packages = [ rustToolchain sccache ] ++ buildInputs ++ cargoTools;

  # Environment variables for the profile script
  envVars = ''
    # Rust source path for rust-analyzer
    export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
    
    # Cargo home
    export CARGO_HOME="$HOME/.cargo"
    export PATH="$CARGO_HOME/bin:$PATH"
    
    # Optional: Use sccache for faster builds
    # Comment out these lines if you want to use incremental compilation instead
    export RUSTC_WRAPPER="${sccache}/bin/sccache"
    export SCCACHE_CACHE_SIZE="10G"
    export SCCACHE_DIR="$HOME/.cache/sccache"
    
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
    
    # Create sccache directory if it doesn't exist
    mkdir -p "$HOME/.cache/sccache"
    
    # Rust-specific optimizations
    # Note: CARGO_INCREMENTAL must be 0 when using sccache
    export CARGO_INCREMENTAL=0
    export RUST_BACKTRACE=1
  '';

  # Setup script for initial configuration
  setupScript = pkgs.writeScript "setup-rust" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    echo "Setting up Rust development environment..."
    
    # Create cargo directories
    mkdir -p "$HOME/.cargo/bin"
    mkdir -p "$HOME/.cache/sccache"
    
    # Create a basic cargo config if it doesn't exist
    if [ ! -f "$HOME/.cargo/config.toml" ]; then
      cat > "$HOME/.cargo/config.toml" <<EOF
    [build]
    # Use sccache for compilation caching
    rustc-wrapper = "${sccache}/bin/sccache"
    
    [target.x86_64-unknown-linux-gnu]
    linker = "${pkgs.gcc}/bin/gcc"
    
    [net]
    git-fetch-with-cli = true
    EOF
    fi
    
    echo "Rust environment setup complete!"
    echo "Rust version: $(${rustToolchain}/bin/rustc --version)"
    echo "Cargo version: $(${rustToolchain}/bin/cargo --version)"
    echo "rust-analyzer available: $(command -v rust-analyzer >/dev/null && echo 'yes' || echo 'no')"
  '';
}
