# Shared dev-shell environment builder
# Copyright Firestream. MIT License.
#
# Extracted VERBATIM from the legacy flake.nix `mkContainerConfig` let-block so
# that both rust.nix (packages.container) and devshell.nix (devShells.default)
# build identical shellPackages / profileScript / container derivations.
#
# Returns: { shellPackages; profileScript; combinedRustToolchain; container; }
{ pkgs, fenix, system }:

let
  # Bitnami charts fetcher (used by the profile script).
  charts = pkgs.fetchgit {
    name = "bitnami-charts";
    url = "https://github.com/bitnami/charts.git";
    rev = "9bc801b4caa0b2fff6ae3392f6b417877a056965";
    sha256 = "8No+rUyEmugs26c7XYo1SAwlafG8sKrhsk6FnaJwL/U=";
    fetchSubmodules = false;
    deepClone = false;
  };

  # Fenix Rust toolchain (deterministic, replaces rustup)
  rustToolchain = fenix.packages.${system}.stable;
  combinedRustToolchain = fenix.packages.${system}.combine [
    rustToolchain.rustc
    rustToolchain.cargo
    rustToolchain.clippy
    rustToolchain.rustfmt
    rustToolchain.rust-src
    rustToolchain.rust-analyzer
  ];

  # Define the shell packages
  shellPackages = with pkgs; [
    # Rust toolchain via Fenix (deterministic)
    combinedRustToolchain

    # Node
    nodejs_22

    # Distribution of protoc and the gRPC Node protoc plugin for ease of installation with npm
    grpc-tools

    # Protobuf and gRPC API tools
    grpcurl
    grpcui

    # Python 3.11
    python311
    python311Packages.pip
    python311Packages.protobuf
    python311Packages.types-protobuf

    # Kafka connector
    rdkafka

    # Java 11 and Maven
    maven
    jdk11

    # Scala
    scala_2_13
    scalafmt
    metals
    sbt-with-scala-native

    # RockDB
    rocksdb

    # Miscellaneous
    curl
    btop

    # LLVM/Clang tools
    llvmPackages_latest.libclang.lib
    llvmPackages.bintools
    clang
    libclang
    llvm
    gcc
    xz
    zlib
    openssl

    # Build Tools
    pkg-config

    # VIB (Validation, Inspection, Build) Tools
    goss    # Server validation and testing
    trivy   # Container vulnerability scanner
    grype   # Vulnerability scanner for containers and filesystems
    docker  # Docker CLI for container management
  ];

  # Create a profile script that sets up the environment
  profileScript = pkgs.writeText "nix-env.sh" ''
    # Add packages to the path
    export PATH="${pkgs.lib.makeBinPath shellPackages}:$PATH"

    # Create symlink to Python in home directory if it doesn't exist
    if [ ! -L "$HOME/.python" ]; then
      ln -sf ${pkgs.python311}/bin/python "$HOME/.python"
    fi

    # Python environment variables
    export PYTHONPATH="$HOME/.python"
    export JUPYTER_PATH="$HOME/.python/share/jupyter"
    export IPYTHONDIR="$HOME/.python"

    # Set up Nix environment
    export NIX_PATH="nixpkgs=${pkgs.path}"
    export NIX_CONFIG="experimental-features = nix-command flakes"

    # Create local charts directory and copy contents
    if [ ! -d "$HOME/bitnami-charts" ]; then
      mkdir -p "$HOME/bitnami-charts"
      cp -r ${charts}/* "$HOME/bitnami-charts/"
      chmod -R u+w "$HOME/bitnami-charts"
    fi
    export BITNAMI_CHARTS_HOME="$HOME/bitnami-charts/"

    # Ensure pkg-config can find system libraries
    export PKG_CONFIG_PATH="${pkgs.xz}/lib/pkgconfig:$PKG_CONFIG_PATH"

    # Try to use system lzma if available
    export LZMA_API_STATIC=1

    # Cargo home for crates.io cache
    export CARGO_HOME="$HOME/.cargo"
    export PATH="$CARGO_HOME/bin:$PATH"

    # Rust source path for IDE integration (Fenix)
    export RUST_SRC_PATH="${combinedRustToolchain}/lib/rustlib/src/rust/library"

    # Bindgen configuration
    export LIBCLANG_PATH="${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ]}"
  '';

  # Container derivation
  container = pkgs.runCommand "container-env"
    {
      buildInputs = [ pkgs.makeWrapper ];
    } ''
    mkdir -p $out/bin $out/etc/profile.d

    # Copy the profile script
    cp ${profileScript} $out/etc/profile.d/nix-env.sh

    # Create the setup script
    cat > $out/bin/setup-container <<EOF
    #!${pkgs.bash}/bin/bash
    set -e

    mkdir -p /etc/profile.d
    mkdir -p /etc/zsh

    cp $out/etc/profile.d/nix-env.sh /etc/profile.d/
    chmod 644 /etc/profile.d/nix-env.sh

    touch /etc/bash.bashrc
    touch /etc/zsh/zshrc

    if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/bash.bashrc; then
      echo '. /etc/profile.d/nix-env.sh' >> /etc/bash.bashrc
    fi

    if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/zsh/zshrc; then
      echo '. /etc/profile.d/nix-env.sh' >> /etc/zsh/zshrc
    fi

    echo "Container environment setup complete!"
    echo "To activate in current shell, run:"
    echo "  source /etc/profile.d/nix-env.sh"
    EOF

    chmod +x $out/bin/setup-container

    # Create symlinks to packages
    mkdir -p $out/packages
    ${pkgs.lib.concatMapStrings (pkg: ''
      ln -s ${pkg} $out/packages/$(basename ${pkg})
    '') shellPackages}
  '';

in
{
  inherit shellPackages profileScript combinedRustToolchain container;
}
