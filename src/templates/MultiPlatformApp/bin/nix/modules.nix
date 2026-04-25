# Unified module system for the development environment
# Manages module registry, directory layout, and module loading logic
{ pkgs, system, fenixPackages ? null, firestreamLib ? null }:

let
  lib = pkgs.lib;

  # Module registry - defines available modules
  moduleRegistry = {
    # Core system packages and environment (always enabled)
    core = {
      path = ./core.nix;
      enabled = true;
    };

    # Python development environment
    python = {
      path = ./python.nix;
      enabled = true;
    };

    # Node.js development environment
    node = {
      path = ./node.nix;
      enabled = true;
    };

    # Rust development environment
    rust = {
      path = ./rust.nix;
      enabled = true;
    };

    # Scala development environment
    scala = {
      path = ./scala.nix;
      enabled = true;
    };

    # Docker-from-Docker support
    docker = {
      path = ./docker.nix;
      enabled = true;
    };

    # Puppeteer/Chromium support
    puppeteer = {
      path = ./puppeteer.nix;
      enabled = true;
    };

    # Tauri development environment
    tauri = {
      path = ./tauri.nix;
      enabled = true;
    };

    # iOS development environment (macOS only)
    ios = {
      path = ./ios.nix;
      enabled = true;
    };

    # Ruby development (for XcodeGen/xcodeproj)
    ruby = {
      path = ./ruby.nix;
      enabled = true;
    };
  };

  # Directory structure definition
  directoryStructure = {
    # User-specific directories (created in user's home)
    userDirs = [
      # Rust directories
      ".cargo"
      ".cargo/bin"
      ".cargo/registry/cache"
      ".cargo/registry/index"
      ".cargo/git"
      ".rustup"

      # Node.js directories
      ".npm-global"
      ".npm-global/bin"
      ".npm-global/lib"
      ".npm-global/lib/node_modules"
      ".npm-cache"

      # Python directories
      ".python"
      ".cache/pip"

      # Scala/Java directories
      ".ivy2"
      ".sbt"
      ".cache/coursier"

      # General development directories
      ".config"
      ".config/nix"
      ".local/bin"
      ".cache"
    ];

    # Workspace directories (created in /workspace)
    workspaceDirs = [
      ".sccache"
      "target"
      "target/debug"
      "target/release"
      ".project-cache"
    ];

    # System directories (only created when running as root)
    systemDirs = [
      "/etc/profile.d"
      "/etc/zsh"
    ];
  };

  # Helper function to create directory creation commands
  mkDirCommands = { dirs, basePath, ownership ? null }:
    lib.concatMapStringsSep "\n" (dir:
      let
        fullPath = if basePath != "" then "${basePath}/${dir}" else dir;
        mkdirCmd = ''mkdir -p "${fullPath}"'';
        chownCmd = if ownership != null then ''chown -R ${ownership} "${fullPath}"'' else "";
      in ''
        ${mkdirCmd}
        ${chownCmd}
      ''
    ) dirs;

  # Create a setup script that creates all directories
  createDirectoriesScript = pkgs.writeScript "create-directories" ''
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Creating directory structure..."

    # Determine the target user
    if [ "$EUID" -eq 0 ]; then
      # Running as root - set up for default user
      SETUP_USER="''${DEFAULT_USER:-developer}"
      SETUP_HOME="/home/$SETUP_USER"
      SETUP_GROUP="$SETUP_USER"

      echo "Running as root, setting up directories for user: $SETUP_USER"

      # Create user directories
      ${mkDirCommands {
        dirs = directoryStructure.userDirs;
        basePath = "$SETUP_HOME";
        ownership = "$SETUP_USER:$SETUP_GROUP";
      }}

      # Create system directories
      ${mkDirCommands {
        dirs = directoryStructure.systemDirs;
        basePath = "";
      }}
    else
      # Running as normal user
      echo "Setting up directories for user: $USER"

      # Create user directories
      ${mkDirCommands {
        dirs = directoryStructure.userDirs;
        basePath = "$HOME";
      }}
    fi

    # Create workspace directories (always)
    ${mkDirCommands {
      dirs = directoryStructure.workspaceDirs;
      basePath = "/workspace";
    }}

    # Set proper permissions on workspace directories
    if [ "$EUID" -eq 0 ]; then
      chown -R "$SETUP_USER:$SETUP_GROUP" /workspace/.sccache /workspace/target /workspace/.project-cache || true
      chmod -R g+w /workspace/.sccache /workspace/target /workspace/.project-cache || true
    fi

    echo "Directory structure created successfully!"
  '';

  # Module loader function
  loadModules = let
    # Filter to only enabled modules
    enabledModules = lib.filterAttrs (_: moduleConfig: moduleConfig.enabled) moduleRegistry;

    # Import and instantiate each enabled module
    loadedModules = lib.mapAttrs (name: moduleConfig:
      import moduleConfig.path {
        inherit pkgs system fenixPackages firestreamLib;
      }
    ) enabledModules;

    # Helper function to safely extract an attribute from modules
    extractFromModules = attrName: defaultValue:
      lib.mapAttrsToList (_: module: module.${attrName} or defaultValue) loadedModules;

    # Helper function to merge module attributes intelligently
    mergeModuleAttribute = attrName: merger: defaultValue:
      let values = extractFromModules attrName defaultValue; in
      if values == [] then defaultValue else merger values;

    # Collect all packages from modules
    allPackages = mergeModuleAttribute "packages" lib.flatten [];

    # Create unified profile script
    unifiedProfileScript = pkgs.writeScript "unified-profile.sh" ''
      # Add all packages to PATH
      export PATH="${lib.makeBinPath allPackages}:$PATH"

      # Add user binaries to PATH
      export PATH="$HOME/.cargo/bin:$HOME/.npm-global/bin:$HOME/.local/bin:$PATH"

      # Configure LD library path
      export LD_LIBRARY_PATH="${lib.makeLibraryPath allPackages}:$LD_LIBRARY_PATH"

      # Module-specific environment setup
      ${mergeModuleAttribute "profileScript" (lib.concatStringsSep "\n\n") ""}
    '';

    # Create unified shell hook
    unifiedShellHook = ''
      # Set up PATH with all packages
      export PATH="${lib.makeBinPath allPackages}:$PATH"
      export PATH="$HOME/.cargo/bin:$HOME/.npm-global/bin:$HOME/.local/bin:$PATH"

      # Configure LD library path
      export LD_LIBRARY_PATH="${lib.makeLibraryPath allPackages}:$LD_LIBRARY_PATH"

      # Module-specific shell setup
      ${mergeModuleAttribute "shellHook" (lib.concatStringsSep "\n\n") ""}
    '';

    # Create master container setup script
    masterSetupScript = pkgs.writeScript "setup-container" ''
      #!/usr/bin/env bash
      set -e

      echo "Setting up container environment..."

      # First, create all required directories
      echo "Creating directory structure..."
      ${createDirectoriesScript}

      # Copy the unified profile script
      mkdir -p /etc/profile.d
      cp ${unifiedProfileScript} /etc/profile.d/nix-env.sh
      chmod 644 /etc/profile.d/nix-env.sh

      # Configure shell integration
      touch /etc/bash.bashrc
      if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/bash.bashrc; then
        echo '. /etc/profile.d/nix-env.sh' >> /etc/bash.bashrc
      fi

      if [ -d /etc/zsh ]; then
        touch /etc/zsh/zshrc
        if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/zsh/zshrc; then
          echo '. /etc/profile.d/nix-env.sh' >> /etc/zsh/zshrc
        fi
      fi

      # Run module setup scripts
      ${lib.concatStringsSep "\n" (
        lib.mapAttrsToList (name: module:
          let setupScript = module.setupScript or null; in
          if setupScript != null then ''
            echo "Running ${name} setup..."
            bash ${setupScript}
          '' else ""
        ) loadedModules
      )}

      echo "Container environment setup complete!"
      echo "To activate in current shell, run:"
      echo "  source /etc/profile.d/nix-env.sh"
    '';

    # Merge all module configurations
    mergedConfig = {
      # All packages from modules
      packages = allPackages;

      # Unified shell hook
      shellHook = unifiedShellHook;

      # Unified profile script as a derivation
      profileScript = unifiedProfileScript;

      # Master setup script
      setupScript = masterSetupScript;

      # Collect apps from all modules
      apps = lib.foldl' (acc: module: acc // (module.apps or {})) {} (lib.attrValues loadedModules);

      # Firestream library for container factories and core libs
      inherit firestreamLib;

      # Keep reference to loaded modules for debugging
      _loadedModules = loadedModules;
      _enabledModules = enabledModules;
    };
  in mergedConfig;

in {
  # Export the module registry for reference
  inherit moduleRegistry directoryStructure;

  # Export the main loader function
  inherit loadModules;
}
