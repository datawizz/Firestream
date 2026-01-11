{
  description = "Firestream JupyterHub - Pure Nix build replacing Bitnami stacksmith dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system (provides fenix/crane for Rust builds)
    firestream.url = "path:../../../..";

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

  outputs = { self, nixpkgs, flake-utils, firestream, pyproject-nix, uv2nix, pyproject-build-systems }:
    let
      inherit (nixpkgs) lib;

      # All supported systems (includes Darwin for package visibility)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # Python version to use
        python = pkgs.python312;

        # JupyterHub version (matching pyproject.toml)
        jupyterhubVersion = "5.3.0";

        # Import Firestream module system via flake input (includes fenix/crane)
        firestreamLib = firestream.firestreamModules { inherit pkgs system; };

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
          # cryptography needs openssl and libffi
          cryptography = prev.cryptography.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.openssl
              pkgs.libffi
            ];
          });

          # psycopg2-binary may need PostgreSQL libs at runtime
          psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.postgresql.lib
            ];
          });

          # pamela needs PAM library
          pamela = prev.pamela.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.linux-pam
            ];
          });
        };

        # 3b. Source build overrides for packages without Linux wheels
        sourceOverrides = final: prev:
          let
            buildPython = pkgs.python312.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ]);
            setupPythonPath = ''
              export PATH="${buildPython}/bin:$PATH"
              export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
            '';
          in {
          # tornado may need source build on some platforms
          tornado = prev.tornado.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # greenlet for async support
          greenlet = prev.greenlet.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
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
            pyproject-build-systems.overlays.default  # Build systems FIRST
            overlay                                    # Workspace overlay second
            sourceOverrides                           # Source build overrides third
            wheelOverrides                            # Runtime overrides last
          ]
        );

        # 6. Create the virtual environment
        pythonEnv = pythonSet.mkVirtualEnv "jupyterhub-env" workspace.deps.default;

        # ============================================================
        # Import the JupyterHub module (Linux only - requires glibc for Docker image)
        # ============================================================
        jupyterhubModule = if isLinux then import ./module.nix {
          inherit pkgs lib pythonEnv jupyterhubVersion python;
          firestream = firestreamLib;
        } else null;

      in {
        # Packages - Docker image only on Linux
        packages = {
          pythonEnv = pythonEnv;
        } // lib.optionalAttrs isLinux {
          default = jupyterhubModule.dockerImage;
          dockerImage = jupyterhubModule.dockerImage;
          runtimeEnv = jupyterhubModule.runtimeEnv;
          entrypoint = jupyterhubModule.scripts.entrypoint;
        };

        # Dev shell - available on all platforms
        devShells.default = if isLinux then jupyterhubModule.devShell else pkgs.mkShell {
          name = "jupyterhub-dev-shell";
          buildInputs = [
            pythonEnv
            pkgs.uv
            pkgs.docker
            pkgs.docker-compose
            pkgs.nodejs_22
          ];
          shellHook = ''
            echo "JupyterHub development shell (macOS)"
            echo "Note: Docker image building requires Linux"
            echo ""
            echo "Available commands:"
            echo "  uv sync         - Sync Python dependencies"
            echo "  python          - Python interpreter with JupyterHub deps"
            echo "  jupyterhub      - Run JupyterHub locally"
          '';
        };

        # Apps for loading Docker image (Linux only)
        apps = lib.optionalAttrs isLinux {
          load-docker = {
            type = "app";
            program = toString (pkgs.writeShellScript "load-docker" ''
              echo "Loading JupyterHub Docker image..."
              docker load < ${jupyterhubModule.dockerImage}
              echo "Image loaded: firestream-jupyterhub:${jupyterhubVersion}-nix"
            '');
          };
        };

        # Export the module for use by other flakes (Linux only)
      } // lib.optionalAttrs isLinux {
        jupyterhubModule = jupyterhubModule;
      });
}
