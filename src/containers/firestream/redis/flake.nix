{
  description = "Firestream Redis Container - Pure Nix build using Firestream module system";

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

        # Import module for each Redis version
        mkRedisContainer = redisVersion: import ./module.nix {
          inherit pkgs lib redisVersion;
          firestream = firestreamLib;
        };

        redis7 = mkRedisContainer "7";
        redis8 = mkRedisContainer "8";
      in {
        packages = {
          default = redis7.dockerImage;
          dockerImage = redis7.dockerImage;
          dockerImage7 = redis7.dockerImage;
          dockerImage8 = redis8.dockerImage;
          runtimeEnv = redis7.runtimeEnv;
          runtimeEnv7 = redis7.runtimeEnv;
          runtimeEnv8 = redis8.runtimeEnv;

          # Expose individual scripts for multi-stage Docker builds
          entrypoint = redis7.scripts.entrypoint;
          entrypoint7 = redis7.scripts.entrypoint;
          entrypoint8 = redis8.scripts.entrypoint;
        };

        devShells.default = redis7.devShell;

        # Expose module for external use
        redisModule7 = redis7;
        redisModule8 = redis8;
      }
    ) // {
      # System-independent outputs for documentation
      lib = {
        # Helper to create Redis containers
        # Note: Callers should use firestream.firestreamModules for proper Rust support
        mkRedisContainer = { pkgs, system, firestreamLib, redisVersion ? "7" }:
          import ./module.nix {
            inherit pkgs redisVersion;
            lib = pkgs.lib;
            firestream = firestreamLib;
          };
      };
    };
}
