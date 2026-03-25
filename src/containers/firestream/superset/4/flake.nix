{
  description = "Firestream Superset - Pure Nix build replacing Bitnami stacksmith dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    # Firestream module system (provides fenix/crane for Rust builds)
    firestream.url = "path:../../../../..";

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

        # Import Firestream module system via flake input (includes fenix/crane)
        firestreamLib = firestream.firestreamModules { inherit pkgs system; };

        # Get wait-for-port package for container runtime
        waitForPortPkg = firestreamLib.waitForPortPkg;

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

          # numba needs Intel TBB for tbbpool threading backend
          numba = prev.numba.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.tbb
              pkgs.stdenv.cc.cc.lib
            ];
          });

          # llvmlite (numba dependency) needs LLVM
          llvmlite = prev.llvmlite.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.llvm
              pkgs.stdenv.cc.cc.lib
            ];
          });
        };

        # 3b. Source build overrides for packages without Linux wheels
        sourceOverrides = final: prev:
          let
            buildPython = pkgs.python311.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ps.hatchling ps.flit-core ]);
            setupPythonPath = ''
              export PATH="${buildPython}/bin:$PATH"
              export PYTHONPATH="${buildPython}/lib/python3.11/site-packages:''${PYTHONPATH:-}"
            '';

            # Helper to add setuptools to a package (use explicit attribute name)
            addSetuptools = pkg: pkg.overrideAttrs (old: {
              nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ buildPython ];
              preBuild = (old.preBuild or "") + setupPythonPath;
            });
          in {
          # python-ldap needs additional system libraries
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

          # All sdist-only packages that need setuptools for building
          # (packages without prebuilt wheels on PyPI)
          apache-superset = addSetuptools prev.apache-superset;
          bottleneck = addSetuptools prev.bottleneck;
          func-timeout = addSetuptools prev.func-timeout;
          python-geohash = addSetuptools prev.python-geohash;
          shortid = addSetuptools prev.shortid;
          alembic = addSetuptools prev.alembic;
          apispec = addSetuptools prev.apispec;
          babel = addSetuptools prev.babel;
          backoff = addSetuptools prev.backoff;
          blinker = addSetuptools prev.blinker;
          cachelib = addSetuptools prev.cachelib;
          cachetools = addSetuptools prev.cachetools;
          celery = addSetuptools prev.celery;
          click = addSetuptools prev.click;
          click-didyoumean = addSetuptools prev.click-didyoumean;
          click-option-group = addSetuptools prev.click-option-group;
          click-plugins = addSetuptools prev.click-plugins;
          click-repl = addSetuptools prev.click-repl;
          colorama = addSetuptools prev.colorama;
          cron-descriptor = addSetuptools prev.cron-descriptor;
          croniter = addSetuptools prev.croniter;
          deprecated = addSetuptools prev.deprecated;
          deprecation = addSetuptools prev.deprecation;
          dnspython = addSetuptools prev.dnspython;
          email-validator = addSetuptools prev.email-validator;
          et-xmlfile = addSetuptools prev.et-xmlfile;
          filelock = addSetuptools prev.filelock;
          flask = addSetuptools prev.flask;
          flask-appbuilder = addSetuptools prev.flask-appbuilder;
          flask-babel = addSetuptools prev.flask-babel;
          flask-caching = addSetuptools prev.flask-caching;
          flask-jwt-extended = addSetuptools prev.flask-jwt-extended;
          flask-limiter = addSetuptools prev.flask-limiter;
          flask-login = addSetuptools prev.flask-login;
          flask-migrate = addSetuptools prev.flask-migrate;
          flask-session = addSetuptools prev.flask-session;
          flask-sqlalchemy = addSetuptools prev.flask-sqlalchemy;
          flask-talisman = addSetuptools prev.flask-talisman;
          flask-wtf = addSetuptools prev.flask-wtf;
          flower = addSetuptools prev.flower;
          future = addSetuptools prev.future;
          geographiclib = addSetuptools prev.geographiclib;
          geopy = addSetuptools prev.geopy;
          google-api-core = addSetuptools prev.google-api-core;
          google-auth = addSetuptools prev.google-auth;
          google-cloud-bigquery = addSetuptools prev.google-cloud-bigquery;
          google-cloud-core = addSetuptools prev.google-cloud-core;
          google-resumable-media = addSetuptools prev.google-resumable-media;
          gunicorn = addSetuptools prev.gunicorn;
          holidays = addSetuptools prev.holidays;
          humanize = addSetuptools prev.humanize;
          idna = addSetuptools prev.idna;
          isodate = addSetuptools prev.isodate;
          itsdangerous = addSetuptools prev.itsdangerous;
          jinja2 = addSetuptools prev.jinja2;
          jmespath = addSetuptools prev.jmespath;
          jsonschema = addSetuptools prev.jsonschema;
          kombu = addSetuptools prev.kombu;
          limits = addSetuptools prev.limits;
          mako = addSetuptools prev.mako;
          markdown = addSetuptools prev.markdown;
          marshmallow = addSetuptools prev.marshmallow;
          marshmallow-sqlalchemy = addSetuptools prev.marshmallow-sqlalchemy;
          ordered-set = addSetuptools prev.ordered-set;
          packaging = addSetuptools prev.packaging;
          prison = addSetuptools prev.prison;
          prompt-toolkit = addSetuptools prev.prompt-toolkit;
          proto-plus = addSetuptools prev.proto-plus;
          protobuf = addSetuptools prev.protobuf;
          pyasn1 = addSetuptools prev.pyasn1;
          pyasn1-modules = addSetuptools prev.pyasn1-modules;
          pyjwt = addSetuptools prev.pyjwt;
          python-dateutil = addSetuptools prev.python-dateutil;
          python-dotenv = addSetuptools prev.python-dotenv;
          pytz = addSetuptools prev.pytz;
          pyyaml = addSetuptools prev.pyyaml;
          redis = addSetuptools prev.redis;
          rich = addSetuptools prev.rich;
          rsa = addSetuptools prev.rsa;
          s3transfer = addSetuptools prev.s3transfer;
          selenium = addSetuptools prev.selenium;
          shortuuid = addSetuptools prev.shortuuid;
          simplejson = addSetuptools prev.simplejson;
          six = addSetuptools prev.six;
          slack-sdk = addSetuptools prev.slack-sdk;
          sqlalchemy = addSetuptools prev.sqlalchemy;
          sqlalchemy-utils = addSetuptools prev.sqlalchemy-utils;
          sqlparse = addSetuptools prev.sqlparse;
          tabulate = addSetuptools prev.tabulate;
          tenacity = addSetuptools prev.tenacity;
          toml = addSetuptools prev.toml;
          tornado = addSetuptools prev.tornado;
          trino = addSetuptools prev.trino;
          typing-extensions = addSetuptools prev.typing-extensions;
          tzdata = addSetuptools prev.tzdata;
          urllib3 = addSetuptools prev.urllib3;
          vine = addSetuptools prev.vine;
          wcwidth = addSetuptools prev.wcwidth;
          werkzeug = addSetuptools prev.werkzeug;
          wrapt = addSetuptools prev.wrapt;
          wtforms = addSetuptools prev.wtforms;
          wtforms-json = addSetuptools prev.wtforms-json;
          xlsxwriter = addSetuptools prev.xlsxwriter;
          zipp = addSetuptools prev.zipp;
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
          inherit pkgs lib pythonEnv supersetVersion python waitForPortPkg;
          firestream = firestreamLib;
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
              echo "Image loaded: firestream-superset:${supersetVersion}"
            '');
          };
        };

        # Export the module for use by other flakes (Linux only)
      } // lib.optionalAttrs isLinux {
        supersetModule = supersetModule;
      });
}
