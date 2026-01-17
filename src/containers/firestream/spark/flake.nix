{
  description = "Firestream Spark - Pure Nix build for Apache Spark container";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system (provides fenix/crane for Rust builds)
    firestream.url = "path:../../../..";
  };

  outputs = { self, nixpkgs, flake-utils, firestream }:
    let
      inherit (nixpkgs) lib;

      # Supported systems (Docker images on Linux, dev shells on all)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # Spark version
        sparkVersion = "4.0.0";

        # Java runtime (Eclipse Temurin JDK 17)
        jdk = pkgs.temurin-bin-17;

        # Python for PySpark
        python = pkgs.python312;

        # Import Firestream module system via flake input (includes fenix/crane)
        firestreamLib = firestream.firestreamModules { inherit pkgs system; };

        # Import the Spark module (Linux only for Docker image)
        sparkModule = if isLinux then import ./module.nix {
          inherit pkgs lib sparkVersion jdk python;
          firestream = firestreamLib;
        } else null;

      in {
        # ============================================================
        # Packages
        # ============================================================
        packages = {
          # Spark package from nixpkgs
          spark = pkgs.spark.override {
            inherit jdk;
            python3 = python;
          };

          # JDK for development
          inherit jdk;

          # Python for PySpark development
          python = python;

        } // lib.optionalAttrs isLinux {
          # Docker image (Linux only)
          default = sparkModule.dockerImage;
          dockerImage = sparkModule.dockerImage;

          # Runtime environment
          runtimeEnv = sparkModule.runtimeEnv;

          # Entrypoint script
          entrypoint = sparkModule.scripts.entrypoint;
        };

        # ============================================================
        # Development Shell
        # ============================================================
        devShells.default = if isLinux then sparkModule.devShell else pkgs.mkShell {
          name = "spark-dev-shell";

          buildInputs = with pkgs; [
            # Java
            jdk
            # Python with PySpark dependencies
            (python.withPackages (ps: with ps; [
              pyspark
              pandas
              numpy
              pyarrow
            ]))
            # Scala/SBT for Spark development
            scala
            sbt
            # Utilities
            curl
            jq
            docker
            docker-compose
          ];

          shellHook = ''
            echo "Spark development shell (${system})"
            ${if !isLinux then ''
            echo "Note: Docker image building requires Linux"
            '' else ""}
            echo ""
            echo "Environment:"
            echo "  Spark: ${sparkVersion}"
            echo "  Java:  ${jdk.name}"
            echo "  Python: ${python.name}"
            echo ""
            echo "Available commands:"
            echo "  spark-shell    - Spark Scala REPL"
            echo "  pyspark        - PySpark REPL"
            echo "  spark-submit   - Submit Spark applications"
            ${if isLinux then ''
            echo ""
            echo "Build commands:"
            echo "  nix build .#dockerImage    - Build the Docker image"
            echo "  docker load < result       - Load image into Docker"
            '' else ""}
          '';

          # Environment variables
          JAVA_HOME = "${jdk}";
          SPARK_HOME = "${pkgs.spark}";
          PYSPARK_PYTHON = "${python}/bin/python3";
        };

        # ============================================================
        # Apps (helper commands)
        # ============================================================
        apps = lib.optionalAttrs isLinux {
          # Load Docker image helper
          load-docker = {
            type = "app";
            program = toString (pkgs.writeShellScript "load-docker" ''
              echo "Loading Spark Docker image..."
              docker load < ${sparkModule.dockerImage}
              echo "Image loaded: firestream-spark:${sparkVersion}"
              echo ""
              echo "Run with:"
              echo "  docker run -e SPARK_MODE=master firestream-spark:${sparkVersion}"
            '');
          };

          # Run Spark master locally
          run-master = {
            type = "app";
            program = toString (pkgs.writeShellScript "run-master" ''
              echo "Starting Spark Master..."
              export JAVA_HOME="${jdk}"
              export SPARK_HOME="${pkgs.spark}"
              exec ${pkgs.spark}/bin/spark-class org.apache.spark.deploy.master.Master
            '');
          };

          # Run Spark worker locally
          run-worker = {
            type = "app";
            program = toString (pkgs.writeShellScript "run-worker" ''
              MASTER_URL="''${1:-spark://localhost:7077}"
              echo "Starting Spark Worker connecting to $MASTER_URL..."
              export JAVA_HOME="${jdk}"
              export SPARK_HOME="${pkgs.spark}"
              exec ${pkgs.spark}/bin/spark-class org.apache.spark.deploy.worker.Worker "$MASTER_URL"
            '');
          };
        };

        # ============================================================
        # Checks (for CI)
        # ============================================================
        checks = lib.optionalAttrs isLinux {
          # Verify the Docker image builds
          docker-image = sparkModule.dockerImage;

          # Verify the dev shell works
          dev-shell = pkgs.runCommand "check-dev-shell" {
            buildInputs = [ jdk python ];
          } ''
            ${jdk}/bin/java -version
            ${python}/bin/python3 --version
            touch $out
          '';
        };

      } // lib.optionalAttrs isLinux {
        # Export the module for use by other flakes
        sparkModule = sparkModule;
      });
}
