{
  description = "Firestream Superset Container - Multi-version support (4.x and 5.x)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system (provides fenix/crane for Rust builds)
    firestream.url = "path:../../../..";

    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, firestream, pyproject-nix, uv2nix, pyproject-build-systems }:
    let
      inherit (nixpkgs) lib;

      # All supported systems (Docker images built on Linux, dev shells on all)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      # Common inputs to pass to version-specific flakes
      commonInputs = {
        inherit nixpkgs flake-utils firestream pyproject-nix uv2nix pyproject-build-systems;
        self = self;
      };
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # Import version-specific flakes and get their outputs
        v4Flake = import ./4/flake.nix;
        v5Flake = import ./5/flake.nix;

        v4Outputs = v4Flake.outputs commonInputs;
        v5Outputs = v5Flake.outputs commonInputs;

        # Helper for unavailable packages on non-Linux
        unavailable = name: pkgs.runCommand "${name}-not-available" {} ''
          echo "Docker images only available on Linux systems" > $out
        '';

      in {
        # Packages - default to v5
        packages = {
          default = if isLinux then v5Outputs.packages.${system}.dockerImage else unavailable "superset";
          dockerImage = if isLinux then v5Outputs.packages.${system}.dockerImage else unavailable "superset";

          # Version-specific packages
          superset-4 = if isLinux then v4Outputs.packages.${system}.dockerImage else unavailable "superset-4";
          superset-5 = if isLinux then v5Outputs.packages.${system}.dockerImage else unavailable "superset-5";
          dockerImage4 = if isLinux then v4Outputs.packages.${system}.dockerImage else unavailable "superset-4";
          dockerImage5 = if isLinux then v5Outputs.packages.${system}.dockerImage else unavailable "superset-5";

          # Python environments (available on all platforms)
          pythonEnv4 = v4Outputs.packages.${system}.pythonEnv;
          pythonEnv5 = v5Outputs.packages.${system}.pythonEnv;
        } // lib.optionalAttrs isLinux {
          # Linux-only exports
          runtimeEnv4 = v4Outputs.packages.${system}.runtimeEnv;
          runtimeEnv5 = v5Outputs.packages.${system}.runtimeEnv;
          entrypoint4 = v4Outputs.packages.${system}.entrypoint;
          entrypoint5 = v5Outputs.packages.${system}.entrypoint;
        };

        # Dev shells - available on all platforms
        devShells = {
          default = v5Outputs.devShells.${system}.default;
          v4 = v4Outputs.devShells.${system}.default;
          v5 = v5Outputs.devShells.${system}.default;
        };

        # Apps for loading Docker images (Linux only)
        apps = lib.optionalAttrs isLinux {
          load-docker = v5Outputs.apps.${system}.load-docker;
          load-docker-4 = v4Outputs.apps.${system}.load-docker;
          load-docker-5 = v5Outputs.apps.${system}.load-docker;
        };
      } // lib.optionalAttrs isLinux {
        # Export modules for use by other flakes
        supersetModule4 = v4Outputs.supersetModule;
        supersetModule5 = v5Outputs.supersetModule;
        supersetModule = v5Outputs.supersetModule;
      });
}
