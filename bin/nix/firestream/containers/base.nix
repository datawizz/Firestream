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
          appModule.scripts.envDefaults
          appModule.scripts.fileLoader
          appModule.scripts.lib
          finalEntrypoint
          appModule.scripts.setup
          appModule.scripts.run
        ] ++ customScriptPackages ++ allRuntimeDeps;
        pathsToLink = [ "/bin" "/lib" "/lib64" "/opt" "/share" "/etc" ];
      };

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
    };

    # Expose app module for advanced usage
    inherit appModule;
  };
}
