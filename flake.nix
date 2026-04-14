{
  description = "Firestream";

  inputs = {
    # Fixed nixpkgs version fixes system packages that float on the latest Debian
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";

    # flake-utils for container flake imports
    flake-utils.url = "github:numtide/flake-utils";

    # Python packaging infrastructure for containers
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

    # gitignore.nix for faster source filtering (respects .gitignore)
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Fenix for deterministic Rust toolchain
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Crane for incremental Rust builds with dependency caching
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, pyproject-nix, uv2nix, pyproject-build-systems, gitignore, fenix, crane }: let
    # Define supported systems (includes Darwin for package visibility)
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
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

    # Filter function to exclude large/unnecessary directories
    # Uses nixpkgs.lib for string functions (available at top level)
    sourceFilter = path: type:
      let
        baseName = baseNameOf (toString path);
        hasSuffix = nixpkgs.lib.hasSuffix;
      in !(
        baseName == ".git" ||
        baseName == "node_modules" ||
        baseName == "result" ||
        baseName == "target" ||
        baseName == "__pycache__" ||
        baseName == ".pytest_cache" ||
        baseName == ".venv" ||
        baseName == ".direnv" ||
        baseName == ".devenv" ||
        hasSuffix ".pyc" baseName ||
        hasSuffix ".egg-info" baseName
      );

    # Filtered source of entire repo - preserves directory structure for relative imports
    # This allows container flakes to use relative paths like ../../../../bin/nix/firestream
    # Uses gitignore.nix for faster filtering (uses git index for clean trees)
    filteredRepoSource = _pkgs: gitignore.lib.gitignoreSource self;

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

      # Fenix Rust toolchain (deterministic, replaces rustup)
      rustToolchain = fenix.packages.${system}.stable;
      combinedRustToolchain = fenix.packages.${system}.combine [
        rustToolchain.rustc
        rustToolchain.cargo
        rustToolchain.clippy
        rustToolchain.rustfmt
        rustToolchain.rust-src
        rustToolchain.rust-analyzer
      ];

      # Define the shell packages
      shellPackages = with pkgs; [

        # Rust toolchain via Fenix (deterministic)
        combinedRustToolchain

        # Node
        nodejs_22

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
        scala_2_13
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

        # Cargo home for crates.io cache
        export CARGO_HOME="$HOME/.cargo"
        export PATH="$CARGO_HOME/bin:$PATH"

        # Rust source path for IDE integration (Fenix)
        export RUST_SRC_PATH="${combinedRustToolchain}/lib/rustlib/src/rust/library"

        # Bindgen configuration
        export LIBCLANG_PATH="${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ]}"
      '';

    in {
      # Shell packages for devShell
      packages = shellPackages;

      # Profile script for sourcing
      inherit profileScript;

      # Combined Rust toolchain
      inherit combinedRustToolchain;

      # Container derivation
      container = pkgs.runCommand "container-env" {
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
    };

  in {
    packages = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      isLinux = builtins.elem system [ "x86_64-linux" "aarch64-linux" ];

      # Helper for unavailable packages on non-Linux
      unavailable = name: pkgs.runCommand "${name}-not-available" {} ''
        echo "Docker images only available on Linux systems" > $out
      '';

      # Import Firestream with Rust module (Fenix + Crane)
      # Also pass Python packaging inputs for mkPythonWorkspaceContainer
      firestreamLib = import ./bin/nix/firestream {
        inherit pkgs system;
        inherit fenix crane;
        inherit pyproject-nix uv2nix pyproject-build-systems;
      };

      # Backward compatibility alias
      firestreamWithRust = firestreamLib;

      # Import PostgreSQL module directly for top-level access
      mkPostgresql = version: let
        firestream = import ./bin/nix/firestream {
          inherit pkgs system;
          inherit fenix crane;
        };
        mod = import ./src/containers/firestream/postgresql/module.nix {
          inherit pkgs firestream version;
          lib = pkgs.lib;
        };
      in mod.dockerImage;

      # Import Airflow using mkPythonWorkspaceContainer
      # Returns full module for fleet manifest access, use .dockerImage for container
      airflowModule = let
        containerPath = ./src/containers/firestream/airflow;
        overrides = import (containerPath + "/overrides.nix") { inherit pkgs; lib = pkgs.lib; };
      in firestreamLib.mkPythonWorkspaceContainer {
        workspacePath = containerPath;
        name = "airflow";
        version = "3.0.3";
        python = pkgs.python312;
        inherit overrides;
      };
      mkAirflow = airflowModule.dockerImage;

      # Import Odoo using mkPythonWorkspaceContainer (multi-version)
      # Returns full module for fleet manifest access, use .dockerImage for container
      # Supported versions: 15, 16, 17, 18
      odooModule = version: let
        containerPath = ./src/containers/firestream/odoo + "/${version}";
        overrides = import (containerPath + "/overrides.nix") { inherit pkgs; lib = pkgs.lib; };

        # Python version per Odoo release
        pythonForVersion = {
          "15" = pkgs.python310;
          "16" = pkgs.python310;
          "17" = pkgs.python311;
          "18" = pkgs.python312;
        }.${version};

        # Pinned source commits per Odoo branch
        odooSrc = pkgs.fetchFromGitHub {
          owner = "odoo";
          repo = "odoo";
          rev = {
            "15" = "3a28e5b0adbb36bdb1155a6854cdfbe4e7f9b187";  # 15.0 branch
            "16" = "f28eb1478fe47c4627fc3377fa505c78a6a7ea82";  # 16.0 branch
            "17" = "92e49e9c5087a4adedf89ff0fbf9462e8487a8da";  # 17.0 branch
            "18" = "f9e25ebf2f22b75ee74742a38182366a3b6a732c";  # 18.0 branch as of 2025-12-23
          }.${version};
          sha256 = {
            "15" = "sha256-nfdElQQIkV/zTVzozRt2CUaGmNPw2G3Zza+feTsBxTs=";
            "16" = "sha256-cXb7TEomwhe0zTEmDhoTy1UcDlQ5mTeyFAdsnhvhBlE=";
            "17" = "sha256-lc1mlEwK3Pgh9bD7lfIHqJ4zjUIheL7DCM8C7tdpf20=";
            "18" = "sha256-XK2+FwV9UDapbl037RR9wRhJZXGIKoHl/1gtB40+g1M=";
          }.${version};
        };

        # Package Odoo source for installation
        odooSource = pkgs.stdenv.mkDerivation {
          pname = "odoo-source";
          version = "${version}.0";
          src = odooSrc;

          installPhase = ''
            mkdir -p $out/opt/odoo
            cp -r . $out/opt/odoo/
            chmod +x $out/opt/odoo/odoo-bin
          '';

          dontBuild = true;
          dontConfigure = true;
          dontFixup = true;
        };
      in firestreamLib.mkPythonWorkspaceContainer {
        workspacePath = containerPath;
        name = "odoo";
        version = "${version}.0";
        python = pythonForVersion;
        inherit overrides odooSource;
      };
      mkOdoo = version: (odooModule version).dockerImage;

      # Import JupyterHub using mkPythonWorkspaceContainer
      # Returns full module for fleet manifest access, use .dockerImage for container
      jupyterhubModule = let
        containerPath = ./src/containers/firestream/jupyterhub;
        overrides = import (containerPath + "/overrides.nix") { inherit pkgs; lib = pkgs.lib; };
      in firestreamLib.mkPythonWorkspaceContainer {
        workspacePath = containerPath;
        name = "jupyterhub";
        version = "5.3.0";
        python = pkgs.python312;
        inherit overrides;
      };
      mkJupyterhub = jupyterhubModule.dockerImage;

      # Import Superset using mkPythonWorkspaceContainer (supports versions 4 and 5)
      # Returns full module for fleet manifest access, use .dockerImage for container
      supersetModule = version: let
        containerPath = ./src/containers/firestream/superset + "/${version}";
        overrides = import (containerPath + "/overrides.nix") { inherit pkgs; lib = pkgs.lib; };
        # Superset uses Python 3.11
      in firestreamLib.mkPythonWorkspaceContainer {
        workspacePath = containerPath;
        name = "superset";
        version = if version == "4" then "4.1.1" else "5";
        python = pkgs.python311;
        inherit overrides;
        # Superset module needs waitForPortPkg
        waitForPortPkg = firestreamLib.waitForPortPkg;
      };
      mkSuperset = version: (supersetModule version).dockerImage;

      # Import Redis module directly for top-level access
      mkRedis = redisVersion: let
        firestream = import ./bin/nix/firestream {
          inherit pkgs system;
          inherit fenix crane;
        };
        mod = import ./src/containers/firestream/redis/module.nix {
          inherit pkgs firestream redisVersion;
          lib = pkgs.lib;
        };
      in mod.dockerImage;

      # Import Kafka module directly for top-level access
      # Returns full module for fleet manifest access, use .dockerImage for container
      kafkaModule = import ./src/containers/firestream/kafka/module.nix {
        inherit pkgs;
        lib = pkgs.lib;
        firestream = firestreamLib;
        version = "4.0";
      };
      mkKafka = kafkaModule.dockerImage;

      # Import Spark module directly for top-level access
      # Returns full module for fleet manifest access, use .dockerImage for container
      sparkModule = import ./src/containers/firestream/spark/module.nix {
        inherit pkgs;
        lib = pkgs.lib;
        firestream = firestreamLib;
        sparkVersion = "4.0.0";
        jdk = pkgs.temurin-bin-17;
        python = pkgs.python312;
      };
      mkSpark = sparkModule.dockerImage;

      config = mkContainerConfig system;
    in {
      container = config.container;
      default = firestreamWithRust.packages.firestream;

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

      # ====================================================================
      # TOP-LEVEL CONTAINER DERIVATIONS
      # Usage: nix build .#postgresql-17
      # ====================================================================
      postgresql-16 = if isLinux then mkPostgresql "16" else unavailable "postgresql-16";
      postgresql-17 = if isLinux then mkPostgresql "17" else unavailable "postgresql-17";
      redis-7 = if isLinux then mkRedis "7" else unavailable "redis-7";
      redis-8 = if isLinux then mkRedis "8" else unavailable "redis-8";
      airflow = if isLinux then mkAirflow else unavailable "airflow";
      airflow-3 = if isLinux then mkAirflow else unavailable "airflow-3";
      odoo = if isLinux then mkOdoo "18" else unavailable "odoo";
      odoo-15 = if isLinux then mkOdoo "15" else unavailable "odoo-15";
      odoo-16 = if isLinux then mkOdoo "16" else unavailable "odoo-16";
      odoo-17 = if isLinux then mkOdoo "17" else unavailable "odoo-17";
      odoo-18 = if isLinux then mkOdoo "18" else unavailable "odoo-18";
      jupyterhub = if isLinux then mkJupyterhub else unavailable "jupyterhub";
      jupyterhub-5 = if isLinux then mkJupyterhub else unavailable "jupyterhub-5";
      superset = if isLinux then mkSuperset "5" else unavailable "superset";
      superset-4 = if isLinux then mkSuperset "4" else unavailable "superset-4";
      superset-5 = if isLinux then mkSuperset "5" else unavailable "superset-5";
      kafka = if isLinux then mkKafka else unavailable "kafka";
      kafka-4 = if isLinux then mkKafka else unavailable "kafka-4";
      spark = if isLinux then mkSpark else unavailable "spark";
      spark-4 = if isLinux then mkSpark else unavailable "spark-4";

      # ====================================================================
      # Container images namespace (backward compatible)
      # Usage: nix build .#containers.airflow
      # ====================================================================
      containers = {
        airflow = if isLinux then mkAirflow else unavailable "airflow";
        airflow-3 = if isLinux then mkAirflow else unavailable "airflow-3";
        odoo = if isLinux then mkOdoo "18" else unavailable "odoo";
        odoo-15 = if isLinux then mkOdoo "15" else unavailable "odoo-15";
        odoo-16 = if isLinux then mkOdoo "16" else unavailable "odoo-16";
        odoo-17 = if isLinux then mkOdoo "17" else unavailable "odoo-17";
        odoo-18 = if isLinux then mkOdoo "18" else unavailable "odoo-18";
        jupyterhub = if isLinux then mkJupyterhub else unavailable "jupyterhub";
        jupyterhub-5 = if isLinux then mkJupyterhub else unavailable "jupyterhub-5";
        superset = if isLinux then mkSuperset "5" else unavailable "superset";
        superset-4 = if isLinux then mkSuperset "4" else unavailable "superset-4";
        superset-5 = if isLinux then mkSuperset "5" else unavailable "superset-5";
        postgresql = if isLinux then mkPostgresql "17" else unavailable "postgresql";
        postgresql-16 = if isLinux then mkPostgresql "16" else unavailable "postgresql-16";
        postgresql-17 = if isLinux then mkPostgresql "17" else unavailable "postgresql-17";
        redis = if isLinux then mkRedis "7" else unavailable "redis";
        redis-7 = if isLinux then mkRedis "7" else unavailable "redis-7";
        redis-8 = if isLinux then mkRedis "8" else unavailable "redis-8";
        kafka = if isLinux then mkKafka else unavailable "kafka";
        kafka-4 = if isLinux then mkKafka else unavailable "kafka-4";
        spark = if isLinux then mkSpark else unavailable "spark";
        spark-4 = if isLinux then mkSpark else unavailable "spark-4";
      };

      # ====================================================================
      # RUST PACKAGES (built with Crane)
      # Usage: nix build .#firestream
      # Usage: nix build .#wait-for-port
      # ====================================================================
      firestream = firestreamWithRust.packages.firestream;
      wait-for-port = firestreamWithRust.packages.wait-for-port;
      firestream-vib = firestreamWithRust.packages.firestream-vib;

      # ====================================================================
      # FLEET MANIFEST - Aggregated SBOM and source archives for all containers and artifacts
      # Usage: nix build .#manifest
      # Outputs: sbom-cyclonedx.json, sbom-spdx.json, manifest.json, source_index.json, sources/
      # ====================================================================
      manifest = if isLinux then
        let
          # Dynamically collect all artifacts tagged for inclusion
          # This includes containers (isFirestreamContainer) and CLI tools (isFirestreamArtifact)
          allArtifacts = firestreamLib.manifest.collectArtifacts {
            # Container modules with metadata
            airflow = airflowModule;
            odoo = odooModule "18";
            odoo-15 = odooModule "15";
            odoo-16 = odooModule "16";
            odoo-17 = odooModule "17";
            jupyterhub = jupyterhubModule;
            superset = supersetModule "5";
            kafka = kafkaModule;
            spark = sparkModule;
            # Rust CLI tools (already wrapped with metadata in packages)
            firestream-tui = firestreamWithRust.packages.firestream;
            wait-for-port = firestreamWithRust.packages.wait-for-port;
            firestream-vib = firestreamWithRust.packages.firestream-vib;
          };
        in firestreamLib.manifest.mkFleetManifest {
          artifacts = allArtifacts;
          fleetName = "firestream";
          version = self.rev or self.dirtyRev or "dev";
          archiveSources = true;
        }
      else unavailable "manifest";

      # ====================================================================
      # INDIVIDUAL SBOM ACCESS
      # Usage: nix build .#sbom.airflow
      # Outputs: metadata.json, sbom-cyclonedx.json, sbom-spdx.json, closure.json
      # ====================================================================
      sbom = if isLinux then {
        airflow = airflowModule.metadata;
        odoo = (odooModule "18").metadata;
        odoo-15 = (odooModule "15").metadata;
        odoo-16 = (odooModule "16").metadata;
        odoo-17 = (odooModule "17").metadata;
        odoo-18 = (odooModule "18").metadata;
        jupyterhub = jupyterhubModule.metadata;
        superset = (supersetModule "5").metadata;
        kafka = kafkaModule.metadata;
        spark = sparkModule.metadata;
        firestream-tui = firestreamWithRust.packages.firestream.metadata;
        wait-for-port = firestreamWithRust.packages.wait-for-port.metadata;
        firestream-vib = firestreamWithRust.packages.firestream-vib.metadata;
      } else {};
    });

    # Development shells with Darwin-specific configuration
    # On Darwin, uses mkShellNoCC to avoid stdenv's automatic SDK setup
    devShells = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      isDarwin = pkgs.stdenv.isDarwin;
      darwin = import ./bin/nix/firestream/modules/darwin.nix {
        inherit pkgs;
        lib = pkgs.lib;
      };
      config = mkContainerConfig system;
    in {
      default = (if isDarwin then pkgs.mkShellNoCC else pkgs.mkShell) {
        packages = config.packages;
        shellHook = darwin.shellHook;
      };
    });

    # Firestream module system library
    # Provides core library functions, environment generation, and application factory
    lib = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      firestream = import ./bin/nix/firestream {
        inherit pkgs system;
        inherit fenix crane;
        inherit pyproject-nix uv2nix pyproject-build-systems;
      };
      firestreamWithRust = firestream;
    in {
      # Complete Firestream module system
      firestream = firestream;

      # Convenience: direct access to factory functions
      mkAppModule = firestream.mkAppModule;
      mkContainerModule = firestream.mkContainerModule;
      mkPythonContainerModule = firestream.mkPythonContainerModule;
      mkPythonWorkspaceContainer = firestream.mkPythonWorkspaceContainer;

      # Core libraries for custom modules
      coreLibs = firestream.coreLibs;

      # Rust module - allows external flakes to build Rust packages
      # Usage: firestream.lib.${system}.rust.mkRustPackage { ... }
      rust = firestreamWithRust.rust;
      mkRustPackage = firestreamWithRust.mkRustPackage;
      rustToolchain = if firestreamWithRust.rust != null
        then firestreamWithRust.rust.toolchain
        else null;
    });

    # Overlay for nixpkgs integration
    # Usage: nixpkgs.overlays = [ firestream.overlays.default ];
    overlays.default = final: prev: {
      firestream = {
        wait-for-port = self.packages.${prev.system}.wait-for-port;
        mkRustPackage = self.lib.${prev.system}.mkRustPackage;
        rustToolchain = self.lib.${prev.system}.rustToolchain;
      };
    };

    # Home Manager module for Firestream CLI/TUI
    # Usage: imports = [ inputs.firestream.homeManagerModules.default ];
    homeManagerModules.default = import ./bin/nix/firestream/home-manager/firestream.nix;
    homeModules.default = import ./bin/nix/firestream/home-manager/firestream.nix;

    # Test suite for the Firestream module system
    # Build with: nix build .#checks.x86_64-linux.firestream-tests
    # Or run all checks: nix flake check
    checks = forAllSystems (system: let
      pkgs = pkgsForSystem system;
      tests = import ./bin/nix/firestream/tests {
        inherit pkgs system;
        inherit fenix crane;
      };
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
    #       modules = firestream.firestreamModules { inherit pkgs system; };
    #     in { ... };
    # }
    firestreamModules = { pkgs, system }: import ./bin/nix/firestream {
      inherit pkgs system;
      inherit fenix crane;
      inherit pyproject-nix uv2nix pyproject-build-systems;
    };

    # Container modules - expose container flake paths for external use
    # Usage from other flakes:
    #   inputs.firestream.url = "path:./path/to/Firestream";
    #   # Then import and call the flake outputs
    containerModulePaths = {
      airflow = "src/containers/firestream/airflow";
      odoo = "src/containers/firestream/odoo";
      jupyterhub = "src/containers/firestream/jupyterhub";
      postgresql = "src/containers/firestream/postgresql";
      # Future containers can be added here:
      # kafka = "src/containers/firestream/kafka";
    };

    # Helper to import container modules from this flake
    # Usage: firestream.mkContainerFromPath { inherit pkgs system; path = firestream.containerModulePaths.airflow; }
    # NOTE: This is deprecated in favor of using mkPythonWorkspaceContainer directly
    # Kept for backward compatibility
    mkContainerFromPath = { pkgs, system, path }:
      let
        lib = pkgs.lib;
        fs = import ./bin/nix/firestream {
          inherit pkgs system;
          inherit fenix crane;
          inherit pyproject-nix uv2nix pyproject-build-systems;
        };
        # Extract container name from path
        containerName = builtins.baseNameOf path;
        containerPath = ./. + "/${path}";
        overridesPath = containerPath + "/overrides.nix";
        hasOverrides = builtins.pathExists overridesPath;
        overrides = if hasOverrides then import overridesPath { inherit pkgs lib; } else {};
        module = fs.mkPythonWorkspaceContainer {
          workspacePath = containerPath;
          name = containerName;
          version = "latest";
          python = pkgs.python312;
          inherit overrides;
        };
      in { inherit (module) dockerImage; default = module.dockerImage; };
  };
}
