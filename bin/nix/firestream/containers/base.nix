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

{ pkgs, lib, mkAppModule, coreLibs, waitForPortPkg, firestreamVibPkg }:

let
  # Import metadata library for SBOM and container metadata generation
  metadataLib = import ../lib/metadata.nix { inherit pkgs lib firestreamVibPkg; };

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
        waitForPortPkg  # Rust-based port checker - available in all containers
      ]
    );

    # Generate container metadata files (SBOM, closure, etc.)
    # These are generated at Nix build time and included in the container image
    containerMetadata = metadataLib.mkContainerMetadata {
      inherit name version;
      mainDrv = appModule.prepopulatedEnv;  # Use prepopulated env as representative derivation
      exposedPorts = exposedPorts;
      user = "${toString user.uid}:${toString user.gid}";
      workdir = paths.base;
      flakeUri = builtins.getEnv "FLAKE_URI";
      flakeRevision = builtins.getEnv "GIT_COMMIT_HASH";
    };

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
      tag = version;
      maxLayers = 100;  # Modern Docker supports 128, use 100 for fine-grained caching

      # Pass as flat list for automatic layer optimization by buildLayeredImage
      # Order: base system → runtime deps → app scripts → config (most to least stable)
      contents = [
        # Layer group 1: System fundamentals (rarely change)
        passwdFile
        groupFile
        shadowFile
        pkgs.bashInteractive
        pkgs.cacert
        pkgs.stdenv.cc.cc.lib
        waitForPortPkg
      ] ++ systemDeps ++ runtimeBinDeps ++ extraDeps ++ [
        # Layer group 2: App scripts (change with code updates)
        appModule.scripts.envDefaults
        appModule.scripts.fileLoader
        appModule.scripts.lib
        finalEntrypoint
        appModule.scripts.setup
        appModule.scripts.run
      ] ++ customScriptPackages ++ [
        # Layer group 3: Config templates (change frequently)
        appModule.prepopulatedEnv
        # Layer group 4: Metadata files (SBOM, closure info - regenerated on each build)
        containerMetadata
      ];

      # Fix ownership for non-root user (runs as fakeroot during image build)
      # Create all runtime directories at build time so container can run as non-root
      # Note: /etc/passwd, /etc/group, /etc/shadow are created as Nix derivations above
      fakeRootCommands = ''
        # Create system /tmp directory with sticky bit (required for mktemp, etc.)
        mkdir -p ./tmp
        chmod 1777 ./tmp

        # CRITICAL: Resolve /etc symlinks to actual files
        # Docker health checks fail with "path escapes from parent" if /etc/* are symlinks
        # to Nix store paths (e.g., /etc/passwd -> /nix/store/xxx-passwd/etc/passwd)
        for etc_file in ./etc/passwd ./etc/group ./etc/shadow; do
          if [ -L "$etc_file" ]; then
            target=$(readlink -f "$etc_file")
            rm "$etc_file"
            cp "$target" "$etc_file"
            echo "Resolved /etc symlink: $etc_file"
          fi
        done

        # Create all runtime directories from runtimeDirs schema
        ${lib.concatMapStringsSep "\n" (dirSpec: ''
          mkdir -p .${dirSpec.path}
          chown ${toString user.uid}:${toString user.gid} .${dirSpec.path}
          chmod ${dirSpec.mode} .${dirSpec.path}
        '') (lib.mapAttrsToList (name: spec: {
          path = spec.path;
          mode = spec.mode or "0755";
        }) (appModule.config.runtimeDirs or {}))}

        # Helper function to resolve symlinks to real files/dirs
        resolve_symlink() {
          local path="$1"
          if [ -L "$path" ]; then
            local target=$(readlink -f "$path")
            rm "$path"
            cp -r "$target" "$path"
            echo "Resolved symlink: $path"
          fi
        }

        # OPTIMIZED: Only resolve app-specific directories (not system paths)
        # /bin, /lib remain as symlinks; /etc is resolved above for health check compatibility
        for base_dir in ./opt ./firestream; do
          app_dir="$base_dir/${name}"
          resolve_symlink "$app_dir"
        done

        # Ensure home directory exists
        mkdir -p ./home/${user.name}

        # OPTIMIZED: Batch ownership with single chown -R call
        chown -R ${toString user.uid}:${toString user.gid} \
          ./opt/${name} \
          ./firestream/${name} \
          ./home/${user.name} \
          2>/dev/null || true

        # OPTIMIZED: Use xargs for batch chmod (10x faster than -exec per file)
        find ./opt/${name} -type d -print0 2>/dev/null | xargs -0 -r chmod 755 2>/dev/null || true
        find ./opt/${name} -type f -print0 2>/dev/null | xargs -0 -r chmod 644 2>/dev/null || true
        find ./firestream/${name} -type d -print0 2>/dev/null | xargs -0 -r chmod 755 2>/dev/null || true
        find ./firestream/${name} -type f -print0 2>/dev/null | xargs -0 -r chmod 644 2>/dev/null || true
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
