{
  description = "Firestream Redis Container - Pure Nix build using Firestream module system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Use relative import (matches PostgreSQL/Airflow pattern)
        firestream = import ../../../../bin/nix/firestream { inherit pkgs; };

        # Import module for each Redis version
        mkRedisContainer = redisVersion: import ./module.nix {
          inherit pkgs lib firestream redisVersion;
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
        mkRedisContainer = { pkgs, redisVersion ? "7" }:
          import ./module.nix {
            inherit pkgs redisVersion;
            lib = pkgs.lib;
            firestream = import ../../../../bin/nix/firestream { inherit pkgs; };
          };
      };
    };
}
