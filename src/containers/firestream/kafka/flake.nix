{
  description = "Firestream Kafka Container - Pure Nix build using Firestream module system";

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

        # Import Kafka module for version 4.0
        kafka = import ./module.nix {
          inherit pkgs lib;
          firestream = firestreamLib;
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
        # Note: Callers should use firestream.firestreamModules for proper Rust support
        mkKafkaContainer = { pkgs, system, firestreamLib, version ? "4.0" }:
          import ./module.nix {
            inherit pkgs version;
            lib = pkgs.lib;
            firestream = firestreamLib;
          };
      };
    };
}
