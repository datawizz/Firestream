{
  description = "Firestream Kafka Container - Pure Nix build using Firestream module system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # Use relative import (matches Firestream module system pattern)
        firestream = import ../../../../bin/nix/firestream { inherit pkgs; };

        # Import Kafka module for version 4.0
        kafka = import ./module.nix {
          inherit pkgs lib firestream;
          version = "4.0";
        };
      in {
        packages = {
          default = kafka.dockerImage;
          dockerImage = kafka.dockerImage;
          runtimeEnv = kafka.runtimeEnv;

          # Expose individual scripts for multi-stage Docker builds
          entrypoint = kafka.scripts.entrypoint;
        };

        devShells.default = kafka.devShell;

        # Expose module for external use
        kafkaModule = kafka;
      }
    ) // {
      # System-independent outputs for documentation
      lib = {
        # Helper to create Kafka containers
        mkKafkaContainer = { pkgs, version ? "4.0" }:
          import ./module.nix {
            inherit pkgs version;
            lib = pkgs.lib;
            firestream = import ../../../../bin/nix/firestream { inherit pkgs; };
          };
      };
    };
}
