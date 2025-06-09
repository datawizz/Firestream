{
  description = "Firestream";

  inputs = {
    # Fixed nixpkgs version fixes system packages that float on the latest Debian
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
  };

  outputs = { self, nixpkgs }: let
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
      rev = "76c10cf217c5f37c4806bfc3b06683b849d8903f";  # Git commit hash
      sha256 = "7kx42VCkuTMCSmJ3lHX48nwL1mri46Ss9R0sJNiWgro=";    # Hash of the files in the repository

      # Optional configurations
      fetchSubmodules = false;  # Set to true if you need submodules
      deepClone = false;        # Keep this false for better performance
    };

    getSparkOperator = pkgs: pkgs.fetchgit {
      name = "spark-operator-charts";
      url = "https://github.com/kubeflow/spark-operator.git";  # Public repository
      rev = "53dc38ef551a7b7ed2c933e14cd5550d74fcee26";  # Git commit hash
      sha256 = "=";    # Hash of the files in the repository

      # Optional configurations
      fetchSubmodules = false;  # Set to true if you need submodules
      deepClone = false;        # Keep this false for better performance
    };

    # Helper function to create a container configuration
    mkContainerConfig = system: let
      pkgs = pkgsForSystem system;
      charts = getBitnamiCharts pkgs;

      # Import Docker-from-Docker module
      dockerInDocker = import ./bin/nix/docker-from-docker.nix { inherit pkgs; };

      # Import Python packages from python.nix
      pythonModule = import ./bin/nix/python.nix {
        inherit pkgs;
        lib = nixpkgs.lib;
        config = {};
      };

      # Extract Python packages from the module
      pythonPackages = pythonModule.environment.systemPackages;

      # Import Rust packages from rust.nix
      rustModule = import ./bin/nix/rust.nix {
        inherit pkgs;
        lib = nixpkgs.lib;
      };

      # Define the shell packages
      shellPackages = with pkgs; [

        # kubernetes
        k3d
        kubectl
        kubernetes-helm


        # Node
        nodejs_22

        # Distribution of protoc and the gRPC Node protoc plugin for ease of installation with npm
        grpc-tools

        # Protobuf and gRPC API tools
        grpcurl
        grpcui

        # Kafka connector
        rdkafka

        # Java 11 and Maven
        maven
        jdk11

        # Spark
        # Version: 3.5.4 ships with Nix 25.05
        spark

        # Scala
        scala_2_13
        scalafmt
        metals
        (sbt.override { jre = jdk11; })

        # RockDB
        rocksdb

        # Miscellaneous
        curl
        btop

        # Additional build tools not included in rust.nix
        # (rust.nix already includes: gcc, clang, llvm, pkg-config, openssl, xz, zlib)
      ] ++ pythonPackages ++ dockerInDocker.packages ++ rustModule.packages;

      # Create a profile script that sets up the environment
      profileScript = pkgs.writeText "nix-env.sh" ''
        # Add packages to the path
        export PATH="${pkgs.lib.makeBinPath shellPackages}:$PATH"

        # Ensure pkg-config can find system libraries
        export PKG_CONFIG_PATH="${pkgs.xz}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

        # C library paths for building Python packages with C extensions
        export C_INCLUDE_PATH="${pkgs.rdkafka.dev}/include:${pkgs.xz.dev}/include:${pkgs.zlib.dev}/include:${pkgs.openssl.dev}/include:$C_INCLUDE_PATH"
        export LIBRARY_PATH="${pkgs.rdkafka}/lib:${pkgs.xz}/lib:${pkgs.zlib}/lib:${pkgs.openssl.out}/lib:$LIBRARY_PATH"
        export PKG_CONFIG_PATH="${pkgs.rdkafka.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

        # OpenSSL specific environment variables for Rust builds
        export OPENSSL_DIR="${pkgs.openssl.dev}"
        export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
        export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

        # Additional build flags for packages that need explicit paths
        export CFLAGS="-I${pkgs.rdkafka.dev}/include -I${pkgs.zlib.dev}/include -I${pkgs.openssl.dev}/include"
        export LDFLAGS="-L${pkgs.rdkafka}/lib -L${pkgs.zlib}/lib -L${pkgs.openssl.out}/lib"

        # Some Python packages look for these specific variables
        export RDKAFKA_INCLUDE_DIR="${pkgs.rdkafka.dev}/include"
        export RDKAFKA_LIB_DIR="${pkgs.rdkafka}/lib"

        # Create symlink to Python in home directory if it doesn't exist
        if [ ! -L "$HOME/.python" ]; then
          ln -sf ${pkgs.python311}/bin/python "$HOME/.python"
        fi

        # Python environment variables
        export PYTHONPATH="$HOME/.python"
        export JUPYTER_PATH="$HOME/.python/share/jupyter"
        export IPYTHONDIR="$HOME/.python"

        # Force UV to use Nix Python
        export UV_PYTHON_DOWNLOADS=never
        export UV_PYTHON="${pkgs.python312}/bin/python"
        export UV_LINK_MODE=copy

        # Auto-detect and use UV when in a project
        alias python='[[ -f pyproject.toml ]] && uv run python || ${pkgs.python312}/bin/python'
        alias pip='[[ -f pyproject.toml ]] && uv pip || ${pkgs.python312}/bin/python -m pip'
        alias jupyter='[[ -f pyproject.toml ]] && uv run jupyter || jupyter'

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

        # Ensure pkg-config can find system libraries (already set above, removing duplicate)
        # export PKG_CONFIG_PATH="${pkgs.xz}/lib/pkgconfig:$PKG_CONFIG_PATH"

        # Try to use system lzma if available
        export LZMA_API_STATIC=1

        # Rust environment from rust.nix
        ${rustModule.envVars}

        # Docker-from-Docker configuration
        ${dockerInDocker.profileScript}
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

        # Copy docker-init.sh to /usr/local/share if it exists in the output
        if [ -f $out/bin/docker-init.sh ]; then
          echo "Installing docker-init.sh..."
          mkdir -p /usr/local/share
          cp -pf $out/bin/docker-init.sh /usr/local/share/docker-init.sh
          chmod 755 /usr/local/share/docker-init.sh 2>/dev/null || true
        fi

        # Run Docker-from-Docker setup if available
        if [ -f ${dockerInDocker.setupDocker}/bin/setup-docker-from-docker ]; then
          echo "Running Docker-from-Docker setup..."
          ${dockerInDocker.setupDocker}/bin/setup-docker-from-docker || {
            echo "Warning: Docker-from-Docker setup failed, but continuing..."
          }
        fi

        # Run Rust setup
        echo "Running Rust environment setup..."
        ${rustModule.setupScript}
        EOF

        chmod +x $out/bin/setup-container

        # Create symlinks to packages
        mkdir -p $out/packages
        ${pkgs.lib.concatMapStrings (pkg: ''
          ln -s ${pkg} $out/packages/$(basename ${pkg})
        '') shellPackages}

        # Add Docker-from-Docker scripts
        mkdir -p $out/bin
        ln -s ${dockerInDocker.dockerInit}/bin/* $out/bin/
        ln -s ${dockerInDocker.setupDocker}/bin/* $out/bin/
      '';

  in {
    packages = forAllSystems (system: {
      container = mkContainerConfig system;
      default = mkContainerConfig system;
    });
  };
}
