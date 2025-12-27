{
  description = "Firestream Airflow - Pure Nix build replacing Bitnami stacksmith dependencies";

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

      # Linux systems for Docker image builds
      linuxSystems = [ "x86_64-linux" "aarch64-linux" ];
      # All systems including macOS for development
      allSystems = linuxSystems ++ [ "x86_64-darwin" "aarch64-darwin" ];
    in
    flake-utils.lib.eachSystem allSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = builtins.elem system linuxSystems;

        # Python version to use (matching Bitnami's python-3.12.11-5)
        python = pkgs.python312;

        # Airflow version (matching Bitnami's airflow-3.0.3-0)
        airflowVersion = "3.0.3";

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
          # Remove docs directory to avoid file collisions
          aiomysql = prev.aiomysql.overrideAttrs (old: {
            postInstall = (old.postInstall or "") + ''
              rm -rf $out/lib/python*/site-packages/docs
            '';
          });

          google-cloud-audit-log = prev.google-cloud-audit-log.overrideAttrs (old: {
            postInstall = (old.postInstall or "") + ''
              rm -rf $out/lib/python*/site-packages/docs
            '';
          });

          grpcio = prev.grpcio.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.zlib
              pkgs.openssl
            ];
          });

          cryptography = prev.cryptography.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.openssl
              pkgs.libffi
            ];
          });

          psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.postgresql.lib
            ];
          });

          lxml = prev.lxml.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libxml2
              pkgs.libxslt
            ];
          });

          mysql-connector-python = prev.mysql-connector-python.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.keyutils.lib
              pkgs.libxcrypt-legacy
              pkgs.krb5
            ];
            autoPatchelfIgnoreMissingDeps = [
              "libudev.so.1"
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
          gssapi = prev.gssapi.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.krb5.dev
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.krb5.lib
            ];
            preBuild = ''
              ${setupPythonPath}
              export PATH="${pkgs.krb5.dev}/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
              export C_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${C_INCLUDE_PATH:-}"
              export CPLUS_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${CPLUS_INCLUDE_PATH:-}"
            '';
            GSSAPI_MAIN_LIB = "${pkgs.krb5.lib}/lib/libgssapi_krb5.so";
            GSSAPI_LINKER_ARGS = "-L${pkgs.krb5.lib}/lib";
            GSSAPI_COMPILER_ARGS = "-DHAS_GSSAPI_EXT_H -I${pkgs.krb5.dev}/include -I${pkgs.krb5.dev}/include/gssapi";
          });

          krb5 = prev.krb5.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.krb5.dev
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.krb5.lib
            ];
            preBuild = ''
              ${setupPythonPath}
              export PATH="${pkgs.krb5.dev}/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
            '';
          });

          pykerberos = prev.pykerberos.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.krb5.dev
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.krb5.lib
            ];
            preBuild = ''
              ${setupPythonPath}
              export PATH="${pkgs.krb5.dev}/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
            '';
          });

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
              export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
              export SETUPTOOLS_USE_DISTUTILS=local
            '';
          });

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

          python-nvd3 = prev.python-nvd3.overrideAttrs (old: {
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
            pyproject-build-systems.overlays.default
            overlay
            sourceOverrides
            wheelOverrides
          ]
        );

        # 6. Create the virtual environment
        pythonEnv = pythonSet.mkVirtualEnv "airflow-env" workspace.deps.default;

        # ============================================================
        # Import the Airflow module
        # ============================================================
        airflowModule = import ./module.nix {
          inherit pkgs lib firestream pythonEnv airflowVersion python;
        };

      in {
        # Packages available on all systems
        packages = {
          inherit pythonEnv;
          inherit (airflowModule) runtimeEnv;
          inherit (airflowModule.scripts) entrypoint;
        } // lib.optionalAttrs isLinux {
          # Docker images only available on Linux
          default = airflowModule.dockerImage;
          dockerImage = airflowModule.dockerImage;
        };

        devShells.default = airflowModule.devShell;

        # Apps only on Linux (Docker required)
        apps = lib.optionalAttrs isLinux {
          load-docker = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "load-docker" ''
              echo "Building and loading Airflow Docker image..."
              nix build .#dockerImage
              docker load < result
              echo "Image loaded: firestream-airflow:${airflowVersion}-nix"
            '';
          };
        };

        # Export the module for use by other flakes
        airflowModule = airflowModule;
      });
}
