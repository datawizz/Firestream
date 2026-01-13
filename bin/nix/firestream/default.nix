{ pkgs, fenix, crane, system }:

let
  lib = pkgs.lib;

  # Import Rust module (Fenix + Crane for incremental builds)
  rustModule = import ./rust { inherit pkgs fenix crane system; };

  # Import custom packages
  packages = import ./packages {
    inherit pkgs lib;
    mkRustPackage = rustModule.mkRustPackage;
  };
  waitForPortPkg = packages.wait-for-port;

  # Import core library modules with dependency injection
  # Each module receives only the dependencies it needs
  logModule = import ./lib/log.nix { inherit pkgs lib; };
  validationsModule = import ./lib/validations.nix { inherit pkgs lib logModule; };
  fsModule = import ./lib/fs.nix { inherit pkgs lib logModule; };
  osModule = import ./lib/os.nix { inherit pkgs lib logModule fsModule validationsModule; };
  netModule = import ./lib/net.nix { inherit pkgs lib logModule validationsModule waitForPortPkg; };
  serviceModule = import ./lib/service.nix { inherit pkgs lib logModule validationsModule; };
  fileModule = import ./lib/file.nix { inherit pkgs lib logModule; };
  persistenceModule = import ./lib/persistence.nix { inherit pkgs lib logModule fsModule; };
  configModule = import ./lib/config.nix { inherit pkgs lib logModule; };
  stateModule = import ./lib/state.nix { inherit pkgs lib logModule fsModule configModule; };
  volumesModule = import ./lib/volumes.nix { inherit pkgs lib logModule fsModule; };

  # Aggregate core libraries into a single attribute set
  coreLibs = {
    log = logModule;
    validations = validationsModule;
    fs = fsModule;
    os = osModule;
    net = netModule;
    service = serviceModule;
    file = fileModule;
    persistence = persistenceModule;
    config = configModule;
    state = stateModule;
    volumes = volumesModule;
  };

  # Import environment modules
  envDefaults = import ./env/defaults.nix { inherit pkgs lib; };
  envFileLoader = import ./env/file-loader.nix { inherit pkgs lib; };

  envModules = {
    mkEnvDefaults = envDefaults.mkEnvDefaults;
    mkFileLoader = envFileLoader.mkFileLoader;
  };

  # Import application factory
  appBase = import ./apps/base.nix { inherit pkgs lib coreLibs envModules; };

  # Import container factories
  containerBase = import ./containers/base.nix {
    inherit pkgs lib coreLibs waitForPortPkg;
    mkAppModule = appBase.mkAppModule;
  };
  containerPython = import ./containers/python.nix {
    inherit pkgs lib;
    mkContainerModule = containerBase.mkContainerModule;
  };
  containerJava = import ./containers/java.nix {
    inherit pkgs lib;
    mkContainerModule = containerBase.mkContainerModule;
  };
  containerNode = import ./containers/node.nix {
    inherit pkgs lib;
    mkContainerModule = containerBase.mkContainerModule;
  };

  containerModules = {
    mkContainerModule = containerBase.mkContainerModule;
    mkPythonContainerModule = containerPython.mkPythonContainerModule;
    mkJavaContainerModule = containerJava.mkJavaContainerModule;
    mkNodeContainerModule = containerNode.mkNodeContainerModule;
  };

  # Import Node.js module (development environment)
  nodeModule = import ./modules/node.nix { inherit pkgs lib; };

  # Import Node.js package builder
  nodeBuilder = import ./node { inherit pkgs lib; };

in {
  # Rust module (Fenix + Crane)
  # Usage: firestream.rust.mkRustPackage { ... }
  # Usage: firestream.rust.toolchain
  rust = rustModule;

  # Convenience: direct access to mkRustPackage
  mkRustPackage = rustModule.mkRustPackage;

  # Custom packages built from source
  # Usage: firestream.packages.wait-for-port
  inherit packages;
  inherit waitForPortPkg;

  # Core library modules (for direct access)
  # Usage: firestream.lib.log.functions
  lib = coreLibs;

  # Environment generation functions
  # Usage: firestream.env.mkEnvDefaults { ... }
  env = envModules;

  # Application module factory
  # Usage: firestream.mkAppModule { name = "kafka"; ... }
  mkAppModule = appBase.mkAppModule;

  # Container module factories
  # Usage: firestream.containers.mkContainerModule { ... }
  # Usage: firestream.containers.mkPythonContainerModule { ... }
  containers = containerModules;

  # Convenience: direct access to container factories
  mkContainerModule = containerModules.mkContainerModule;
  mkPythonContainerModule = containerModules.mkPythonContainerModule;
  mkJavaContainerModule = containerModules.mkJavaContainerModule;
  mkNodeContainerModule = containerModules.mkNodeContainerModule;

  # Node.js module (development environment)
  # Usage: firestream.node.packages, firestream.node.shellHook
  node = nodeModule;

  # Node.js package builder
  # Usage: firestream.mkNodePackage { pname = "my-app"; src = ./.; ... }
  mkNodePackage = nodeBuilder.mkNodePackage;

  # Convenience: combined functions from all core libs
  # Returns a single string containing all library functions
  allFunctions = lib.concatMapStringsSep "\n" (m: m.functions) (lib.attrValues coreLibs);

  # Convenience: all runtime deps from core libs
  # Returns a deduplicated list of all runtime dependencies
  allRuntimeDeps = lib.unique (lib.concatMap (m: m.runtimeDeps) (lib.attrValues coreLibs));

  # Convenience: generate a combined library script
  # Creates a single bash script file with all library functions
  combinedLibScript = pkgs.writeTextDir "opt/firestream/scripts/libfirestream.sh" ''
    #!/bin/bash
    # Combined Firestream library functions
    # Copyright Firestream. Apache-2.0 License.
    # Generated by Nix - do not edit directly.

    ${lib.concatMapStringsSep "\n\n" (m: m.functions) (lib.attrValues coreLibs)}
  '';

  # Meta information about the module system
  meta = {
    name = "firestream-nix-modules";
    version = "1.0.0";
    description = "Firestream Nix Shell Module System for container initialization";
    moduleCount = builtins.length (lib.attrNames coreLibs);
    modules = lib.attrNames coreLibs;
    containerFactories = [ "mkContainerModule" "mkPythonContainerModule" "mkJavaContainerModule" "mkNodeContainerModule" ];
    packageBuilders = [ "mkRustPackage" "mkNodePackage" ];
    devModules = [ "node" ];
  };
}
