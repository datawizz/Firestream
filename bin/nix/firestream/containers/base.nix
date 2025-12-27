# Container Module Factory
# Copyright Firestream. Apache-2.0 License.
#
# This module provides mkContainerModule - a factory function that generates
# complete container modules with Docker image building capabilities.
#
# It wraps mkAppModule with Docker-specific concerns:
# - Docker image building via dockerTools.buildLayeredImage
# - Development shell generation
# - Runtime environment bundling
#
# Usage:
#   let
#     container = mkContainerModule {
#       name = "myapp";
#       version = "1.0.0";
#       systemDeps = [ pkgs.curl pkgs.git ];
#       runtimeBinDeps = [ pkgs.coreutils ];
#       validateFn = "...";
#       configFn = "...";
#       initFn = "...";
#       runCmd = "myapp --serve";
#     };
#   in container.dockerImage

{ pkgs, lib, mkAppModule, coreLibs }:

let
  # Default Docker configuration
  defaultDockerConfig = {
    WorkingDir = "/opt/firestream";
    User = "1001:1001";
  };

  # Default exposed ports
  defaultExposedPorts = {};

  # Default volumes
  defaultVolumes = {
    "/firestream/data" = {};
  };

in
{
  # Factory function to create container modules
  mkContainerModule = {
    # Basic metadata
    name,
    version ? "1.0.0",

    # Environment configuration (passed to mkAppModule)
    envVars ? {},
    envVarsWithSecrets ? [],

    # Paths configuration
    paths ? {
      base = "/opt/firestream/${name}";
      conf = "/opt/firestream/${name}/config";
      data = "/firestream/${name}/data";
      logs = "/opt/firestream/${name}/logs";
    },

    # User/group configuration
    user ? {
      name = name;
      group = name;
      uid = 1001;
      gid = 1001;
    },

    # Application-specific shell functions
    validateFn ? "",
    configFn ? "",
    initFn ? "",
    runCmd ? "",

    # Build-time (prepopulate phase) - passed to mkAppModule
    prepopulateFn ? "",
    prepopulateFiles ? {},
    prepopulateDirs ? [],
    runtimeDirs ? {},              # Declarative directory specifications (preferred)

    # Runtime (activate phase) - passed to mkAppModule
    activateFn ? "",
    enableStateTracking ? true,

    # Container-specific dependencies
    systemDeps ? [],          # System packages for the container (libs, tools)
    runtimeBinDeps ? [],      # Packages that need to be in PATH at runtime
    extraDeps ? [],           # Additional deps for mkAppModule

    # Docker configuration
    dockerConfig ? {},        # Override Docker image config
    exposedPorts ? [],        # List of port numbers to expose
    volumes ? [],             # List of volume paths

    # Optional custom entrypoint wrapper
    entrypointWrapper ? null, # Function: entrypoint -> wrapped entrypoint

    # Optional additional scripts
    customScripts ? {},       # { name = script; } - additional scripts to include

    # Development shell extras
    devShellPackages ? [],    # Extra packages for dev shell
    devShellHook ? "",        # Additional shell hook for dev shell
  }:
  let
    # Create the application module
    appModule = mkAppModule {
      inherit name version envVars envVarsWithSecrets paths user;
      inherit validateFn configFn initFn runCmd;
      inherit prepopulateFn prepopulateFiles prepopulateDirs runtimeDirs;
      inherit activateFn enableStateTracking;
      extraDeps = extraDeps ++ runtimeBinDeps;
    };

    # Combine all runtime dependencies
    allRuntimeDeps = lib.lists.unique (
      appModule.runtimeDeps ++ systemDeps ++ runtimeBinDeps ++ [
        pkgs.bashInteractive
        pkgs.cacert
        pkgs.stdenv.cc.cc.lib
      ]
    );

    # Build exposed ports config
    portsConfig = lib.listToAttrs (
      map (port: {
        name = "${toString port}/tcp";
        value = {};
      }) exposedPorts
    );

    # Build volumes config
    volumesConfig = lib.listToAttrs (
      map (path: {
        name = path;
        value = {};
      }) ([ paths.data paths.logs ] ++ volumes)
    );

    # Get the entrypoint, optionally wrapped
    finalEntrypoint =
      if entrypointWrapper != null
      then entrypointWrapper appModule.scripts.entrypoint
      else appModule.scripts.entrypoint;

    # Build custom scripts
    customScriptPackages = lib.mapAttrsToList (
      scriptName: scriptContent:
        pkgs.writeScriptBin scriptName scriptContent
    ) customScripts;

    # Create /etc/passwd as a proper Nix derivation (idiomatic approach)
    passwdFile = pkgs.writeTextDir "etc/passwd" ''
root:x:0:0:root:/root:/bin/bash
nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin
${user.name}:x:${toString user.uid}:${toString user.gid}:${user.name}:/home/${user.name}:/bin/bash
'';

    # Create /etc/group as a proper Nix derivation
    groupFile = pkgs.writeTextDir "etc/group" ''
root:x:0:
nobody:x:65534:
${user.group}:x:${toString user.gid}:
'';

    # Create /etc/shadow for completeness (locked passwords)
    shadowFile = pkgs.writeTextDir "etc/shadow" ''
root:!x:::::::
nobody:!x:::::::
${user.name}:!:::::::
'';

    # Complete runtime environment
    runtimeEnv = pkgs.buildEnv {
      name = "${name}-runtime-env";
      paths = [
        appModule.scripts.envDefaults
        appModule.scripts.fileLoader
        appModule.scripts.lib
        finalEntrypoint
        appModule.scripts.setup
        appModule.scripts.run
      ] ++ customScriptPackages ++ allRuntimeDeps;
      pathsToLink = [ "/bin" "/lib" "/lib64" "/opt" "/share" "/etc" ];
    };

    # Docker image
    dockerImage = pkgs.dockerTools.buildLayeredImage {
      name = "firestream-${name}";
      tag = "${version}-nix";

      contents = pkgs.buildEnv {
        name = "${name}-root";
        paths = [
          passwdFile        # /etc/passwd as Nix derivation
          groupFile         # /etc/group as Nix derivation
          shadowFile        # /etc/shadow as Nix derivation
          appModule.prepopulatedEnv
          appModule.scripts.envDefaults
          appModule.scripts.fileLoader
          appModule.scripts.lib
          finalEntrypoint
          appModule.scripts.setup
          appModule.scripts.run
        ] ++ customScriptPackages ++ allRuntimeDeps;
        pathsToLink = [ "/bin" "/lib" "/lib64" "/opt" "/firestream" "/share" "/etc" ];
      };

      # Fix ownership for non-root user (runs as fakeroot during image build)
      # Create all runtime directories at build time so container can run as non-root
      # Note: /etc/passwd, /etc/group, /etc/shadow are created as Nix derivations above
      fakeRootCommands = ''
        # Create all runtime directories from runtimeDirs schema
        ${lib.concatMapStringsSep "\n" (dirSpec: ''
          mkdir -p .${dirSpec.path}
          chown ${toString user.uid}:${toString user.gid} .${dirSpec.path}
          chmod ${dirSpec.mode} .${dirSpec.path}
        '') (lib.mapAttrsToList (name: spec: {
          path = spec.path;
          mode = spec.mode or "0755";
        }) (appModule.config.runtimeDirs or {}))}

        # CRITICAL FIX: Replace symlink trees with real directory copies
        # buildEnv creates symlinks to Nix store which are immutable.
        # We must copy the entire tree to make it writable by the runtime user.
        # This handles both:
        # 1. Entire directories that are symlinks (e.g., ./opt/airflow -> /nix/store/...)
        # 2. Individual files that are symlinks within real directories
        for base_dir in ./opt ./firestream; do
          app_dir="$base_dir/${name}"
          if [ -L "$app_dir" ]; then
            # Directory is a symlink - resolve and copy entire tree
            target=$(readlink -f "$app_dir")
            rm "$app_dir"
            cp -r "$target" "$app_dir"
            echo "Copied symlink tree: $app_dir -> $target"
          elif [ -d "$app_dir" ]; then
            # Directory exists - recursively resolve any symlinks within
            find "$app_dir" -type l 2>/dev/null | while read -r symlink; do
              target=$(readlink -f "$symlink" 2>/dev/null || true)
              if [ -e "$target" ]; then
                rm "$symlink"
                if [ -d "$target" ]; then
                  cp -r "$target" "$symlink"
                else
                  cp "$target" "$symlink"
                fi
              fi
            done
          fi
        done

        # Handle /etc symlinks (passwd, group, shadow from Nix derivations)
        # These need to be real files for proper permissions
        if [ -d "./etc" ]; then
          find ./etc -type l 2>/dev/null | while read -r symlink; do
            target=$(readlink -f "$symlink" 2>/dev/null || true)
            if [ -f "$target" ]; then
              rm "$symlink"
              cp "$target" "$symlink"
              chmod 644 "$symlink"
            fi
          done
        fi

        # Ensure home directory exists
        mkdir -p ./home/${user.name}

        # Set ownership recursively (works now because everything is real files/dirs)
        chown -R ${toString user.uid}:${toString user.gid} ./opt/${name} 2>/dev/null || true
        chown -R ${toString user.uid}:${toString user.gid} ./firestream/${name} 2>/dev/null || true
        chown -R ${toString user.uid}:${toString user.gid} ./home/${user.name} 2>/dev/null || true

        # Set appropriate permissions
        # Directories: 755 (owner can write, others can traverse)
        # Files: 644 (owner can write, others can read)
        find ./opt/${name} -type d -exec chmod 755 {} \; 2>/dev/null || true
        find ./opt/${name} -type f -exec chmod 644 {} \; 2>/dev/null || true
        find ./firestream/${name} -type d -exec chmod 755 {} \; 2>/dev/null || true
        find ./firestream/${name} -type f -exec chmod 644 {} \; 2>/dev/null || true
      '';

      # Enable fakeroot for ownership changes
      enableFakechroot = true;

      config = lib.recursiveUpdate (lib.recursiveUpdate defaultDockerConfig {
        Env = [
          "PATH=/bin:/usr/bin"
          "HOME=/home/${user.name}"
          "LD_LIBRARY_PATH=/lib:/lib64:${pkgs.stdenv.cc.cc.lib}/lib"
          "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          "NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        ];
        WorkingDir = paths.base;
        Entrypoint = [ "${finalEntrypoint}/bin/${name}-entrypoint" ];
        ExposedPorts = lib.recursiveUpdate defaultExposedPorts portsConfig;
        Volumes = lib.recursiveUpdate defaultVolumes volumesConfig;
        User = "${toString user.uid}:${toString user.gid}";
      }) dockerConfig;
    };

    # Development shell
    devShell = pkgs.mkShell {
      buildInputs = allRuntimeDeps ++ devShellPackages ++ [
        pkgs.docker
        pkgs.docker-compose
      ];

      shellHook = ''
        echo "Firestream ${name} Development Environment"
        echo "Version: ${version}"
        echo ""
        echo "Available commands:"
        echo "  nix build .#dockerImage    - Build the Docker image"
        echo "  nix develop               - Enter development shell"
        echo ""
        ${devShellHook}
      '';
    };

  in {
    # Module metadata
    meta = {
      inherit name version;
      description = "Firestream ${name} container module";
      maintainer = "Firestream Contributors";
      license = "Apache-2.0";
    };

    # All runtime dependencies
    runtimeDeps = allRuntimeDeps;

    # Generated scripts (from appModule plus any custom)
    scripts = appModule.scripts // {
      custom = customScriptPackages;
    };

    # Complete runtime environment
    inherit runtimeEnv;

    # Docker image
    inherit dockerImage;

    # Development shell
    inherit devShell;

    # Configuration used (for introspection)
    config = appModule.config // {
      inherit systemDeps runtimeBinDeps exposedPorts volumes;
      inherit prepopulateFn prepopulateFiles prepopulateDirs runtimeDirs activateFn enableStateTracking;
    };

    # Volume hints for orchestration (from appModule)
    inherit (appModule) volumeHints;

    # Expose prepopulated environment and state library from appModule
    inherit (appModule) prepopulatedEnv stateLib;

    # Expose app module for advanced usage
    inherit appModule;
  };
}
