{
  description = "Firestream Superset - Pure Nix build replacing Bitnami stacksmith dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";

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
  };

  outputs = { self, nixpkgs, flake-utils, pyproject-nix, uv2nix, pyproject-build-systems }:
    let
      inherit (nixpkgs) lib;

      # All supported systems (Docker images built on Linux, dev shells on all)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # Python version - Superset 4.1.x requires Python 3.11
        python = pkgs.python311;

        # Superset version (matching pyproject.toml)
        supersetVersion = "4.1.1";

        # Import Firestream module system (relative path for standalone builds)
        firestream = import ../../../../bin/nix/firestream { inherit pkgs; };

        # ============================================================
        # uv2nix Integration - Properly resolve all Python dependencies
        # ============================================================

        # 1. Load workspace from uv.lock
        workspace = uv2nix.lib.workspace.loadWorkspace {
          workspaceRoot = ./.;
        };

        # 2. Create overlay preferring binary wheels
        overlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel";
        };

        # 3. Wheel runtime overrides for C extensions needing system libs
        wheelOverrides = final: prev: {
          # Cryptography needs openssl and libffi
          cryptography = prev.cryptography.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.openssl
              pkgs.libffi
            ];
          });

          # psycopg2-binary needs postgresql
          psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.postgresql.lib
            ];
          });

          # lxml needs libxml2/libxslt
          lxml = prev.lxml.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libxml2
              pkgs.libxslt
            ];
          });

          # gevent needs libevent
          gevent = prev.gevent.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libevent
            ];
          });

          # Pillow needs image libraries
          pillow = prev.pillow.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libjpeg
              pkgs.zlib
              pkgs.libpng
              pkgs.freetype
            ];
          });

          # pyarrow needs arrow-cpp
          pyarrow = prev.pyarrow.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.arrow-cpp
            ];
          });

          # grpcio needs zlib and openssl
          grpcio = prev.grpcio.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.zlib
              pkgs.openssl
            ];
          });

          # markupsafe needs gcc runtime
          markupsafe = prev.markupsafe.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.stdenv.cc.cc.lib
            ];
          });
        };

        # 3b. Source build overrides for packages without Linux wheels
        sourceOverrides = final: prev:
          let
            buildPython = pkgs.python311.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ]);
            setupPythonPath = ''
              export PATH="${buildPython}/bin:$PATH"
              export PYTHONPATH="${buildPython}/lib/python3.11/site-packages:''${PYTHONPATH:-}"
            '';
          in {
          # python-ldap needs openldap
          python-ldap = prev.python-ldap.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.openldap.dev
              pkgs.cyrus_sasl.dev
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.openldap
              pkgs.cyrus_sasl
            ];
            postPatch = ''
              ${pkgs.gnused}/bin/sed -i 's/compile = 1/compile = 0/g' setup.cfg
              ${pkgs.gnused}/bin/sed -i 's/optimize = 1/optimize = 0/g' setup.cfg
            '';
            preBuild = ''
              export PYTHONPATH="${buildPython}/lib/python3.11/site-packages:''${PYTHONPATH:-}"
              export SETUPTOOLS_USE_DISTUTILS=local
            '';
          });

          # mysqlclient needs libmysqlclient
          mysqlclient = prev.mysqlclient.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libmysqlclient
              pkgs.openssl
              pkgs.zlib
            ];
            preBuild = setupPythonPath;
          });
        };

        # 4. Create base Python package set
        pythonBase = pkgs.callPackage pyproject-nix.build.packages {
          inherit python;
        };

        # 5. Compose all overlays - ORDER MATTERS
        pythonSet = pythonBase.overrideScope (
          lib.composeManyExtensions [
            pyproject-build-systems.overlays.default
            overlay
            sourceOverrides
            wheelOverrides
          ]
        );

        # 6. Create the virtual environment
        pythonEnv = pythonSet.mkVirtualEnv "superset-env" workspace.deps.default;

        # ============================================================
        # Import the Superset module (Linux only - requires glibc for Docker image)
        # ============================================================
        supersetModule = if isLinux then import ./module.nix {
          inherit pkgs lib firestream pythonEnv supersetVersion python;
        } else null;

      in {
        # Packages - Docker image only on Linux
        packages = {
          pythonEnv = pythonEnv;
        } // lib.optionalAttrs isLinux {
          default = supersetModule.dockerImage;
          dockerImage = supersetModule.dockerImage;
          runtimeEnv = supersetModule.runtimeEnv;
          entrypoint = supersetModule.scripts.entrypoint;
        };

        # Dev shell - available on all platforms
        devShells.default = if isLinux then supersetModule.devShell else pkgs.mkShell {
          name = "superset-dev-shell";
          buildInputs = [
            pythonEnv
            pkgs.uv
            pkgs.docker
            pkgs.docker-compose
          ];
          shellHook = ''
            echo "Superset development shell (macOS)"
            echo "Note: Docker image building requires Linux"
            echo ""
            echo "Available commands:"
            echo "  uv sync         - Sync Python dependencies"
            echo "  python          - Python interpreter with Superset"
          '';
        };

        # Apps for loading Docker image (Linux only)
        apps = lib.optionalAttrs isLinux {
          load-docker = {
            type = "app";
            program = toString (pkgs.writeShellScript "load-docker" ''
              echo "Loading Superset Docker image..."
              docker load < ${supersetModule.dockerImage}
              echo "Image loaded: firestream-superset:${supersetVersion}-nix"
            '');
          };
        };

        # Export the module for use by other flakes (Linux only)
      } // lib.optionalAttrs isLinux {
        supersetModule = supersetModule;
      });
}
