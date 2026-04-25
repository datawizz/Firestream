{
  description = "MultiPlatformApp Development Environment";

  inputs = {
    # Fixed nixpkgs version for reproducible builds
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";

    # Fenix for Rust toolchain with cross-compilation targets
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # flake-utils for eachDefaultSystem helper
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system - container factories, core libs, overlay
    # Pinned to the exact commit this template was generated from
    firestream = {
      url = "github:datawizz/Firestream/446821f0d69da2e3a1ef6d9f54065a5c3eb880da";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, flake-utils, firestream }:
    let
      # Define supported systems
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      # Helper function to generate system-specific attributes
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Configure nixpkgs
      nixpkgsConfig = {
        config = {
          allowUnfree = true;
        };
      };

      # Create properly configured pkgs for each system with Firestream overlay
      pkgsForSystem = system:
        import nixpkgs {
          inherit system;
          inherit (nixpkgsConfig) config;
          overlays = [ firestream.overlays.default ];
        };

      # Import the unified module system
      moduleSystem = import ./bin/nix/modules.nix;

      # Create system-specific configurations (evaluated once per system)
      mkSystemConfig = system:
        let
          pkgs = pkgsForSystem system;

          # Load modules using the unified module system
          loadedModules = (moduleSystem {
            inherit pkgs system;
            fenixPackages = fenix.packages.${system};
            firestreamLib = firestream.lib.${system};
          }).loadModules;

          # Development shell
          devShell = pkgs.mkShell {
            buildInputs = loadedModules.packages;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            shellHook = loadedModules.shellHook;

            # Environment variables for development
            RUST_BACKTRACE = 1;
            LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
          };

          # Container environment configuration
          container = pkgs.runCommand "container-env" {
            buildInputs = [ pkgs.makeWrapper ];
          } ''
            mkdir -p $out/bin $out/etc/profile.d $out/packages

            # Copy the unified profile script
            cp ${loadedModules.profileScript} $out/etc/profile.d/nix-env.sh
            chmod 644 $out/etc/profile.d/nix-env.sh

            # Copy the master setup script
            cp ${loadedModules.setupScript} $out/bin/setup-container
            chmod +x $out/bin/setup-container

            # Copy all module test and utility scripts
            ${nixpkgs.lib.concatStringsSep "\n" (
              nixpkgs.lib.mapAttrsToList (appName: app: ''
                echo "Copying app ${appName}..."
                cp ${app.program} $out/bin/${appName} || echo "Failed to copy ${appName}"
                chmod +x $out/bin/${appName} || true
              '') loadedModules.apps
            )}

            # Create symlinks to packages
            ${nixpkgs.lib.concatMapStrings (pkg: ''
              BASENAME=$(basename ${pkg})
              if [ ! -e "$out/packages/$BASENAME" ]; then
                ln -s ${pkg} $out/packages/$BASENAME
              fi
            '') loadedModules.packages}
          '';

        in
        {
          inherit devShell;
          devcontainer = container;
          # Expose loaded modules for debugging
          _modules = loadedModules;
        };

      # Memoize: evaluate mkSystemConfig once per system
      configs = forAllSystems mkSystemConfig;

    in
    {
      packages = forAllSystems (system: {
        devcontainer = configs.${system}.devcontainer;
        default = configs.${system}.devcontainer;
      });

      devShells = forAllSystems (system: {
        default = configs.${system}.devShell;
      });

      # All apps come from modules
      apps = forAllSystems (system: configs.${system}._modules.apps);

      # Re-export Firestream lib for downstream use
      lib = forAllSystems (system: {
        firestream = firestream.lib.${system};
      });
    };
}
