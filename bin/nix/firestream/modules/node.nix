# bin/nix/firestream/modules/node.nix
# Node.js development environment module
#
# Provides a complete Node.js development environment with:
# - Node.js 22 (configurable)
# - pnpm as primary package manager (npm as fallback)
# - TypeScript support with ts-node
# - Home directory global package configuration
#
# Module Interface (ConceptDB-compatible):
# - packages: List of Nix packages to include
# - shellHook: Shell script snippet for interactive shells
# - profileScript: Shell script for persistent environment setup
# - setupScript: Initialization script for containers
# - apps: CLI applications
#
# Usage:
#   let
#     nodeModule = import ./modules/node.nix { inherit pkgs lib; };
#   in {
#     devShells.default = pkgs.mkShell {
#       packages = nodeModule.packages;
#       shellHook = nodeModule.shellHook;
#     };
#   }

{ pkgs, lib, nodejs ? pkgs.nodejs_22 }:

let
  # TypeScript tooling
  typescript = pkgs.nodePackages.typescript;
  tsNode = pkgs.nodePackages.ts-node;

  # Create pnpm configuration script
  configurePnpmScript = pkgs.writeScriptBin "configure-pnpm" ''
    #!${pkgs.bash}/bin/bash
    set -e

    # Create directory for global pnpm packages
    mkdir -p "$HOME/.pnpm-global"
    mkdir -p "$HOME/.pnpm-store"

    # Configure pnpm to use home directory
    ${pkgs.nodePackages.pnpm}/bin/pnpm config set global-dir "$HOME/.pnpm-global"
    ${pkgs.nodePackages.pnpm}/bin/pnpm config set store-dir "$HOME/.pnpm-store"
    ${pkgs.nodePackages.pnpm}/bin/pnpm config set global-bin-dir "$HOME/.pnpm-global/bin"

    echo "pnpm configured successfully"
  '';

  # Create npm configuration script
  configureNpmScript = pkgs.writeScriptBin "configure-npm" ''
    #!${pkgs.bash}/bin/bash
    set -e

    # Create directory for global npm packages
    mkdir -p "$HOME/.npm-global"
    mkdir -p "$HOME/.npm-cache"

    # Configure npm to use home directory for global packages
    ${nodejs}/bin/npm config set prefix "$HOME/.npm-global"
    ${nodejs}/bin/npm config set cache "$HOME/.npm-cache"

    echo "npm configured successfully"
  '';

  # Environment variables for Node.js development
  envVars = ''
    # Node.js version info
    export NODE_VERSION="${nodejs.version}"

    # pnpm configuration (primary package manager)
    export PNPM_HOME="$HOME/.pnpm-global"
    export PATH="$PNPM_HOME/bin:$PATH"

    # npm configuration (fallback)
    export NPM_CONFIG_PREFIX="$HOME/.npm-global"
    export NPM_CONFIG_CACHE="$HOME/.npm-cache"
    export PATH="$HOME/.npm-global/bin:$PATH"

    # Node.js module resolution
    export NODE_PATH="$HOME/.pnpm-global/lib/node_modules:$HOME/.npm-global/lib/node_modules"

    # TypeScript configuration
    export TS_NODE_TRANSPILE_ONLY=true
  '';

  # Profile script for persistent environment setup (containers)
  profileScript = ''
    ${envVars}

    # Initialize pnpm on first use if not configured
    if [ ! -d "$HOME/.pnpm-global" ]; then
      mkdir -p "$HOME/.pnpm-global"
      mkdir -p "$HOME/.pnpm-store"
      mkdir -p "$HOME/.pnpm-global/bin"

      if command -v pnpm &> /dev/null; then
        pnpm config set global-dir "$HOME/.pnpm-global" 2>/dev/null || true
        pnpm config set store-dir "$HOME/.pnpm-store" 2>/dev/null || true
        pnpm config set global-bin-dir "$HOME/.pnpm-global/bin" 2>/dev/null || true
      fi
    fi

    # Initialize npm on first use if not configured
    if [ ! -d "$HOME/.npm-global" ]; then
      mkdir -p "$HOME/.npm-global"
      mkdir -p "$HOME/.npm-cache"

      if command -v npm &> /dev/null; then
        npm config set prefix "$HOME/.npm-global" 2>/dev/null || true
        npm config set cache "$HOME/.npm-cache" 2>/dev/null || true
      fi
    fi
  '';

  # Setup script for container initialization
  setupScript = pkgs.writeScript "setup-node" ''
    #!${pkgs.bash}/bin/bash
    set -e

    echo "Initializing Node.js environment..."

    # Configure pnpm
    ${configurePnpmScript}/bin/configure-pnpm

    # Configure npm
    ${configureNpmScript}/bin/configure-npm

    echo "Node.js environment initialized"
  '';

  # Shell hook for development
  shellHook = ''
    ${envVars}

    # Display Node.js environment info
    echo "Node.js: $(node --version)"
    echo "pnpm: $(pnpm --version 2>/dev/null || echo 'not configured')"
    echo "TypeScript: $(tsc --version 2>/dev/null || echo 'not installed')"
  '';

in {
  # Module metadata
  meta = {
    name = "node";
    description = "Node.js development environment with pnpm and TypeScript";
    nodeVersion = nodejs.version;
    primaryPackageManager = "pnpm";
  };

  # Packages to include in the environment
  packages = [
    nodejs
    pkgs.nodePackages.pnpm
    typescript
    tsNode
    configurePnpmScript
    configureNpmScript
  ];

  # Environment variables (for use in other modules)
  inherit envVars;

  # Shell hook for development environments
  inherit shellHook;

  # Profile script for persistent environment (containers)
  inherit profileScript;

  # Setup script for initialization
  inherit setupScript;

  # CLI applications
  apps = {
    configure-pnpm = {
      type = "app";
      program = "${configurePnpmScript}/bin/configure-pnpm";
    };
    configure-npm = {
      type = "app";
      program = "${configureNpmScript}/bin/configure-npm";
    };
    setup-node = {
      type = "app";
      program = "${setupScript}";
    };
  };

  # Export individual components for flexibility
  inherit nodejs typescript tsNode;
  inherit configurePnpmScript configureNpmScript;
}
