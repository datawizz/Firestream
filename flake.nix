{
  description = "Firestream";

  inputs = {
    # Fixed nixpkgs version fixes system packages that float on the latest Debian
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";

    # Container flakes - each can be built independently or via root
    airflow-container = {
      url = "path:./src/containers/firestream/airflow";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, airflow-container }: let
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

    #TODO include google protobufs as a library on a local path for build time
    # Pass "PROTO_HOME" to the proto build steps.
    # For now copying the proto files to the local directory
    # https://github.com/protocolbuffers/protobuf/tree/main/src/google/protobuf

    # Add the Bitnami charts fetcher
    getBitnamiCharts = pkgs: pkgs.fetchgit {
      name = "bitnami-charts";
      url = "https://github.com/bitnami/charts.git";  # Public repository
      rev = "9bc801b4caa0b2fff6ae3392f6b417877a056965";  # Git commit hash
      sha256 = "8No+rUyEmugs26c7XYo1SAwlafG8sKrhsk6FnaJwL/U=";    # Hash of the files in the repository

      # Optional configurations
      fetchSubmodules = false;  # Set to true if you need submodules
      deepClone = false;        # Keep this false for better performance
    };

    # Helper function to create a container configuration
    mkContainerConfig = system: let
      pkgs = pkgsForSystem system;
      charts = getBitnamiCharts pkgs;

      # Define the shell packages
      shellPackages = with pkgs; [

        # Rust toolchain
        # rustc
        # cargo
        # rust-analyzer
        # rustfmt
        rustup
        # TODO rustup makes things non-deterministic. Normal Nix install doesn't include linked C libraries and struggles with rust-src required for linting


        # Node
        nodejs_18

        # Distribution of protoc and the gRPC Node protoc plugin for ease of installation with npm
        grpc-tools

        # Protobuf and gRPC API tools
        grpcurl
        grpcui

        # Python 3.11
        python311
        python311Packages.pip
        python311Packages.protobuf
        python311Packages.types-protobuf

        # Kafka connector
        rdkafka

        # Java 11 and Maven
        maven
        jdk11

        # Scala
        scala_2_11
        scalafmt
        metals
        sbt-with-scala-native

        # RockDB
        rocksdb

        # Miscellaneous
        curl
        btop

        # LLVM/Clang tools
        llvmPackages_latest.libclang.lib
        llvmPackages.bintools
        clang
        libclang
        llvm
        gcc
        xz
        zlib
        openssl

        # Build Tools
        pkg-config

        # VIB (Validation, Inspection, Build) Tools
        # These tools are used for container testing and vulnerability scanning
        goss    # Server validation and testing
        trivy   # Container vulnerability scanner
        grype   # Vulnerability scanner for containers and filesystems
        docker  # Docker CLI for container management
      ];

      # Create a profile script that sets up the environment
      profileScript = pkgs.writeText "nix-env.sh" ''
        # Add packages to the path
        export PATH="${pkgs.lib.makeBinPath shellPackages}:$PATH"

        # Create symlink to Python in home directory if it doesn't exist
        if [ ! -L "$HOME/.python" ]; then
          ln -sf ${pkgs.python311}/bin/python "$HOME/.python"
        fi

        # Python environment variables
        export PYTHONPATH="$HOME/.python"
        export JUPYTER_PATH="$HOME/.python/share/jupyter"
        export IPYTHONDIR="$HOME/.python"

        # Set up Nix environment
        export NIX_PATH="nixpkgs=${pkgs.path}"
        export NIX_CONFIG="experimental-features = nix-command flakes"

        # Create local charts directory and copy contents
        if [ ! -d "$HOME/bitnami-charts" ]; then
          mkdir -p "$HOME/bitnami-charts"
          cp -r ${charts}/* "$HOME/bitnami-charts/"
          chmod -R u+w "$HOME/bitnami-charts"
        fi
        export BITNAMI_CHARTS_HOME="$HOME/bitnami-charts/"

        # Ensure pkg-config can find system libraries
        export PKG_CONFIG_PATH="${pkgs.xz}/lib/pkgconfig:$PKG_CONFIG_PATH"

        # Try to use system lzma if available
        export LZMA_API_STATIC=1

        # rustup environment
        export PATH="$HOME/.cargo/bin:$PATH"

        # Set up Rust source path
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"

        # Bindgen configuration
        export LIBCLANG_PATH="${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ]}"
      '';

    in pkgs.runCommand "container-env" {
        buildInputs = [ pkgs.makeWrapper ];
      } ''
        mkdir -p $out/bin $out/etc/profile.d

        # Copy the profile script
        cp ${profileScript} $out/etc/profile.d/nix-env.sh

        # Create the setup script
        cat > $out/bin/setup-container <<EOF
        #!${pkgs.bash}/bin/bash
        set -e

        mkdir -p /etc/profile.d
        mkdir -p /etc/zsh

        cp $out/etc/profile.d/nix-env.sh /etc/profile.d/
        chmod 644 /etc/profile.d/nix-env.sh

        touch /etc/bash.bashrc
        touch /etc/zsh/zshrc

        if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/bash.bashrc; then
          echo '. /etc/profile.d/nix-env.sh' >> /etc/bash.bashrc
        fi

        if ! grep -q '. /etc/profile.d/nix-env.sh' /etc/zsh/zshrc; then
          echo '. /etc/profile.d/nix-env.sh' >> /etc/zsh/zshrc
        fi

        echo "Container environment setup complete!"
        echo "To activate in current shell, run:"
        echo "  source /etc/profile.d/nix-env.sh"
        EOF

        chmod +x $out/bin/setup-container

        # Create symlinks to packages
        mkdir -p $out/packages
        ${pkgs.lib.concatMapStrings (pkg: ''
          ln -s ${pkg} $out/packages/$(basename ${pkg})
        '') shellPackages}
      '';

  in {
    packages = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      isLinux = builtins.elem system [ "x86_64-linux" "aarch64-linux" ];
    in {
      container = mkContainerConfig system;
      default = mkContainerConfig system;

      # VIB (Validation, Inspection, Build) Tools bundle
      # Collection of tools for container testing and vulnerability scanning
      vib-tools = pkgs.buildEnv {
        name = "firestream-vib-tools";
        paths = [
          pkgs.goss    # Server validation and testing
          pkgs.trivy   # Container vulnerability scanner
          pkgs.grype   # Vulnerability scanner for containers and filesystems
          pkgs.docker  # Docker CLI for container management
        ];
      };

      # Container images namespace
      # Usage: nix build .#containers.airflow
      containers = {
        # Airflow container (Docker image only on Linux)
        airflow = if isLinux
          then airflow-container.packages.${system}.dockerImage
          else pkgs.runCommand "airflow-not-available" {} ''
            echo "Docker images only available on Linux systems" > $out
          '';
      };
    });

    # Firestream module system library
    # Provides core library functions, environment generation, and application factory
    lib = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      firestream = import ./bin/nix/firestream { inherit pkgs; };
    in {
      # Complete Firestream module system
      firestream = firestream;

      # Convenience: direct access to mkAppModule
      mkAppModule = firestream.mkAppModule;
    });

    # Test suite for the Firestream module system
    # Build with: nix build .#checks.x86_64-linux.firestream-tests
    # Or run all checks: nix flake check
    checks = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      tests = import ./bin/nix/firestream/tests { inherit pkgs; };
    in {
      firestream-tests = tests.all;

      # Individual test modules (for granular testing)
      firestream-log-tests = tests.logTests;
      firestream-validations-tests = tests.validationsTests;
      firestream-fs-tests = tests.fsTests;
      firestream-os-tests = tests.osTests;
      firestream-net-tests = tests.netTests;
      firestream-service-tests = tests.serviceTests;
      firestream-file-tests = tests.fileTests;
      firestream-persistence-tests = tests.persistenceTests;
      firestream-integration-tests = tests.integrationTests;
      firestream-config-tests = tests.configTests;
      firestream-container-tests = tests.containerTests;
    });

    # System-independent Firestream module system
    # For importing from other flakes:
    #
    # {
    #   inputs.firestream.url = "path:./path/to/Firestream";
    #   outputs = { firestream, nixpkgs, ... }:
    #     let
    #       modules = firestream.firestreamModules { inherit pkgs; };
    #     in { ... };
    # }
    firestreamModules = { pkgs }: import ./bin/nix/firestream { inherit pkgs; };

    # Container modules - expose container flakes for external use
    # Usage from other flakes:
    #   inputs.firestream.url = "path:./path/to/Firestream";
    #   airflowImage = firestream.containerModules.airflow.packages.x86_64-linux.dockerImage;
    containerModules = {
      airflow = airflow-container;
      # Future containers can be added here:
      # postgresql = postgresql-container;
      # kafka = kafka-container;
    };
  };
}
