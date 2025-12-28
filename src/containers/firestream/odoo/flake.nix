{
  description = "Firestream Odoo - Pure Nix build replacing Bitnami stacksmith dependencies";

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

      # All supported systems (includes Darwin for package visibility)
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # Python version to use (matching Bitnami's python-3.12.11-5)
        python = pkgs.python312;

        # Odoo version (matching Bitnami's odoo-18.0)
        odooVersion = "18.0";

        # Import Firestream module system (relative path for standalone builds)
        firestream = import ../../../../bin/nix/firestream { inherit pkgs; };

        # Fetch Odoo source from GitHub (pinned to specific commit for reproducibility)
        odooSrc = pkgs.fetchFromGitHub {
          owner = "odoo";
          repo = "odoo";
          rev = "f9e25ebf2f22b75ee74742a38182366a3b6a732c";  # 18.0 branch as of 2025-12-23
          sha256 = "sha256-XK2+FwV9UDapbl037RR9wRhJZXGIKoHl/1gtB40+g1M=";
        };

        # Package Odoo source for installation
        odooSource = pkgs.stdenv.mkDerivation {
          pname = "odoo-source";
          version = odooVersion;
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
          # lxml needs libxml2 and libxslt
          lxml = prev.lxml.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libxml2
              pkgs.libxslt
            ];
          });

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

          # Pillow needs image libraries
          pillow = prev.pillow.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libjpeg
              pkgs.libpng
              pkgs.zlib
              pkgs.freetype
            ];
          });

          # reportlab for PDF generation
          reportlab = prev.reportlab.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.freetype
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
          # python-ldap needs OpenLDAP and SASL libraries
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

          # gevent needs special handling
          gevent = prev.gevent.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libev
              pkgs.libuv
            ];
            preBuild = setupPythonPath;
          });

          # libsass for SCSS compilation
          libsass = prev.libsass.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libsass
            ];
            preBuild = setupPythonPath;
          });

          # docopt - legacy package needs setuptools
          docopt = prev.docopt.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # ofxparse - legacy package needs setuptools
          ofxparse = prev.ofxparse.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # vobject - legacy package needs setuptools
          vobject = prev.vobject.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # xlwt - legacy package needs setuptools
          xlwt = prev.xlwt.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # polib - legacy package needs setuptools
          polib = prev.polib.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # ebaysdk - legacy package needs setuptools
          ebaysdk = prev.ebaysdk.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
            ];
            preBuild = setupPythonPath;
          });

          # rjsmin - legacy package needs setuptools
          rjsmin = prev.rjsmin.overrideAttrs (old: {
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
        pythonEnv = pythonSet.mkVirtualEnv "odoo-env" workspace.deps.default;

        # ============================================================
        # Import the Odoo module (Linux only - requires glibc for Docker image)
        # ============================================================
        odooModule = if isLinux then import ./module.nix {
          inherit pkgs lib firestream pythonEnv odooVersion python odooSource;
        } else null;

      in {
        # Packages - Docker image only on Linux
        packages = {
          pythonEnv = pythonEnv;
          odooSource = odooSource;
        } // lib.optionalAttrs isLinux {
          default = odooModule.dockerImage;
          dockerImage = odooModule.dockerImage;
          runtimeEnv = odooModule.runtimeEnv;
          entrypoint = odooModule.scripts.entrypoint;
        };

        # Dev shell - available on all platforms
        devShells.default = if isLinux then odooModule.devShell else pkgs.mkShell {
          name = "odoo-dev-shell";
          buildInputs = [
            pythonEnv
            pkgs.uv
            pkgs.docker
            pkgs.docker-compose
          ];
          shellHook = ''
            echo "Odoo development shell (macOS)"
            echo "Note: Docker image building requires Linux"
            echo ""
            echo "Available commands:"
            echo "  uv sync         - Sync Python dependencies"
            echo "  python          - Python interpreter with Odoo deps"
          '';
        };

        # Apps for loading Docker image (Linux only)
        apps = lib.optionalAttrs isLinux {
          load-docker = {
            type = "app";
            program = toString (pkgs.writeShellScript "load-docker" ''
              echo "Loading Odoo Docker image..."
              docker load < ${odooModule.dockerImage}
              echo "Image loaded: firestream-odoo:${odooVersion}-nix"
            '');
          };
        };

        # Export the module for use by other flakes (Linux only)
      } // lib.optionalAttrs isLinux {
        odooModule = odooModule;
      });
}
