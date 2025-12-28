{
  description = "Firestream PostgreSQL Container - Pure Nix build using Firestream module system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Use relative import (matches Airflow pattern)
        firestream = import ../../../../bin/nix/firestream { inherit pkgs; };

        # Import module for each PostgreSQL version
        mkPostgresqlContainer = version: import ./module.nix {
          inherit pkgs lib firestream version;
        };

        pg16 = mkPostgresqlContainer "16";
        pg17 = mkPostgresqlContainer "17";
      in {
        packages = {
          default = pg17.dockerImage;
          dockerImage = pg17.dockerImage;
          dockerImage16 = pg16.dockerImage;
          dockerImage17 = pg17.dockerImage;
          runtimeEnv = pg17.runtimeEnv;
          runtimeEnv16 = pg16.runtimeEnv;
          runtimeEnv17 = pg17.runtimeEnv;

          # Expose individual scripts for multi-stage Docker builds
          entrypoint = pg17.scripts.entrypoint;
          entrypoint16 = pg16.scripts.entrypoint;
          entrypoint17 = pg17.scripts.entrypoint;
        };

        devShells.default = pg17.devShell;

        # Expose module for external use
        postgresqlModule16 = pg16;
        postgresqlModule17 = pg17;
      }
    ) // {
      # System-independent outputs for documentation
      lib = {
        # Helper to create PostgreSQL containers
        mkPostgresqlContainer = { pkgs, version ? "17" }:
          import ./module.nix {
            inherit pkgs version;
            lib = pkgs.lib;
            firestream = import ../../../../bin/nix/firestream { inherit pkgs; };
          };
      };
    };
}
