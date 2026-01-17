{
  description = "Firestream PostgreSQL Container - Pure Nix build using Firestream module system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system (provides fenix/crane for Rust builds)
    firestream.url = "path:../../../..";
  };

  outputs = { self, nixpkgs, flake-utils, firestream }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Import Firestream module system via flake input (includes fenix/crane)
        firestreamLib = firestream.firestreamModules { inherit pkgs system; };

        # Import module for each PostgreSQL version
        mkPostgresqlContainer = version: import ./module.nix {
          inherit pkgs lib version;
          firestream = firestreamLib;
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
        # Note: Callers should use firestream.firestreamModules for proper Rust support
        mkPostgresqlContainer = { pkgs, system, firestreamLib, version ? "17" }:
          import ./module.nix {
            inherit pkgs version;
            lib = pkgs.lib;
            firestream = firestreamLib;
          };
      };
    };
}
