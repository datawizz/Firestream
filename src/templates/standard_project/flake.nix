{
  description = "Standard Project Development Environment";

  inputs = {
    # Fixed nixpkgs version for reproducible builds
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
  };

  outputs = { self, nixpkgs }:
    let
      # Define supported systems
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      # Helper function to generate system-specific attributes
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Configure nixpkgs
      nixpkgsConfig = {
        config = {
          allowUnfree = true;
        };
      };

      # Create properly configured pkgs for each system
      pkgsForSystem = system:
        import nixpkgs {
          inherit system;
          inherit (nixpkgsConfig) config;
        };

      # Import the unified module system
      moduleSystem = import ./bin/nix/modules.nix;

      # Create system-specific configurations
      mkSystemConfig = system:
        let
          pkgs = pkgsForSystem system;

          # Load modules using the unified module system
          loadedModules = (moduleSystem { inherit pkgs self system; }).loadModules;

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
          inherit devShell container;
          # Expose loaded modules for debugging
          _modules = loadedModules;
        };

    in
    {
      packages = forAllSystems (system: {
        container = (mkSystemConfig system).container;
        default = (mkSystemConfig system).container;
      });

      devShells = forAllSystems (system: {
        default = (mkSystemConfig system).devShell;
      });

      # All apps come from modules
      apps = forAllSystems (system:
        let
          config = mkSystemConfig system;
        in
        config._modules.apps
      );
    };
}
