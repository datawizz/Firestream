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
      firestreamWithRust = import ./bin/nix/firestream {
        inherit pkgs system;
        inherit fenix crane;
      };

      # Import PostgreSQL module directly for top-level access
      mkPostgresql = version: let
        firestream = import ./bin/nix/firestream { inherit pkgs; };
        mod = import ./src/containers/firestream/postgresql/module.nix {
          inherit pkgs firestream version;
          lib = pkgs.lib;
        };
      in mod.dockerImage;

      # Import Airflow (requires Python packaging inputs)
      mkAirflow = let
        repoSrc = filteredRepoSource pkgs;
        airflowFlake = import "${repoSrc}/src/containers/firestream/airflow/flake.nix";
        airflowOutputs = airflowFlake.outputs {
          inherit self nixpkgs flake-utils pyproject-nix uv2nix pyproject-build-systems;
        };
      in airflowOutputs.packages.${system}.dockerImage;

      # Import Odoo (requires Python packaging inputs)
      mkOdoo = let
        repoSrc = filteredRepoSource pkgs;
        odooFlake = import "${repoSrc}/src/containers/firestream/odoo/flake.nix";
        odooOutputs = odooFlake.outputs {
          inherit self nixpkgs flake-utils pyproject-nix uv2nix pyproject-build-systems;
        };
      in odooOutputs.packages.${system}.dockerImage;

      # Import JupyterHub (requires Python packaging inputs)
      mkJupyterhub = let
        repoSrc = filteredRepoSource pkgs;
        jupyterhubFlake = import "${repoSrc}/src/containers/firestream/jupyterhub/flake.nix";
        jupyterhubOutputs = jupyterhubFlake.outputs {
          inherit self nixpkgs flake-utils pyproject-nix uv2nix pyproject-build-systems;
        };
      in jupyterhubOutputs.packages.${system}.dockerImage;

      # Import Redis module directly for top-level access
      mkRedis = redisVersion: let
        firestream = import ./bin/nix/firestream { inherit pkgs; };
        mod = import ./src/containers/firestream/redis/module.nix {
          inherit pkgs firestream redisVersion;
          lib = pkgs.lib;
        };
      in mod.dockerImage;

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
      odoo = if isLinux then mkOdoo else unavailable "odoo";
      jupyterhub = if isLinux then mkJupyterhub else unavailable "jupyterhub";

      # ====================================================================
      # Container images namespace (backward compatible)
      # Usage: nix build .#containers.airflow
      # ====================================================================
      containers = {
        airflow = if isLinux then mkAirflow else unavailable "airflow";
        odoo = if isLinux then mkOdoo else unavailable "odoo";
        jupyterhub = if isLinux then mkJupyterhub else unavailable "jupyterhub";
        postgresql = if isLinux then mkPostgresql "17" else unavailable "postgresql";
        postgresql-16 = if isLinux then mkPostgresql "16" else unavailable "postgresql-16";
        postgresql-17 = if isLinux then mkPostgresql "17" else unavailable "postgresql-17";
        redis = if isLinux then mkRedis "7" else unavailable "redis";
        redis-7 = if isLinux then mkRedis "7" else unavailable "redis-7";
        redis-8 = if isLinux then mkRedis "8" else unavailable "redis-8";
      };

      # ====================================================================
      # RUST PACKAGES (built with Crane)
      # Usage: nix build .#firestream
      # Usage: nix build .#wait-for-port
      # ====================================================================
      firestream = firestreamWithRust.packages.firestream;
      wait-for-port = firestreamWithRust.packages.wait-for-port;
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
      firestream = import ./bin/nix/firestream { inherit pkgs; };
      firestreamWithRust = import ./bin/nix/firestream {
        inherit pkgs system;
        inherit fenix crane;
      };
    in {
      # Complete Firestream module system
      firestream = firestream;

      # Convenience: direct access to factory functions
      mkAppModule = firestream.mkAppModule;
      mkContainerModule = firestream.mkContainerModule;
      mkPythonContainerModule = firestream.mkPythonContainerModule;

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
    #       modules = firestream.firestreamModules { inherit pkgs system; };
    #     in { ... };
    # }
    firestreamModules = { pkgs, system }: import ./bin/nix/firestream {
      inherit pkgs system;
      inherit fenix crane;
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
    mkContainerFromPath = { pkgs, system, path }:
      let
        repoSrc = filteredRepoSource pkgs;
        containerFlake = import "${repoSrc}/${path}/flake.nix";
        outputs = containerFlake.outputs {
          inherit self nixpkgs flake-utils pyproject-nix uv2nix pyproject-build-systems;
        };
      in outputs.packages.${system} or {};
  };
}
