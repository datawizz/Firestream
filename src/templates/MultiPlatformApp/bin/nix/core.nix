# core.nix - Core system packages and environment
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Core packages that are always needed
  corePackages = with pkgs; [
    # Version control
    git
    git-lfs
    openssh

    # Build tools
    gcc
    gnumake
    pkg-config
    binutils
    cmake

    # System libraries
    zlib
    openssl
    xz

    # Utilities
    curl
    wget
    btop
    tree
    jq
    ripgrep
    fd

    # Container tools
    docker-client
    docker-compose
  ];

  # Core environment setup
  coreProfileScript = ''
    # Nix environment
    export NIX_PATH="nixpkgs=${pkgs.path}"
    export NIX_CONFIG="experimental-features = nix-command flakes"

    # System library configuration
    export PKG_CONFIG_PATH="${pkgs.xz}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LZMA_API_STATIC=1

    # OpenSSL configuration
    export OPENSSL_DIR="${pkgs.openssl.dev}"
    export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
    export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
  '';

  # Core shell hook
  coreShellHook = ''
    ${coreProfileScript}
  '';

in
{
  # Standardized module interface
  packages = corePackages;
  shellHook = coreShellHook;
  profileScript = coreProfileScript;
  setupScript = null; # No special setup needed for core

  # No apps for core module
  apps = {};
}
