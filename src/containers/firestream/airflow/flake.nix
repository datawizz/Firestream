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

        # System dependencies that were in Bitnami's Debian packages
        systemDeps = with pkgs; [
          # SSL/TLS
          cacert
          openssl

          # Kerberos
          krb5

          # LDAP
          openldap
          cyrus_sasl

          # Database clients
          postgresql
          mariadb-connector-c

          # Compression
          bzip2
          xz
          zlib

          # Version control
          git
          openssh

          # Process management
          procps

          # Utilities
          coreutils
          gnugrep
          gnused
          gawk
          findutils
          which
          curl
          wget
          netcat-gnu

          # Terminal
          ncurses
          readline
        ];

        # Runtime binaries needed by Airflow for PATH injection
        runtimeBinDeps = with pkgs; [
          postgresql       # psql client for DB operations
          git              # For DAG sync from git repos
          coreutils
          gnugrep
          gnused
          findutils
          curl
          netcat-gnu
          openssh          # For SSH operators
        ];

        # Replacement for Bitnami's wait-for-port utility
        waitForPort = pkgs.writeShellScriptBin "wait-for-port" ''
          #!/usr/bin/env bash
          set -e

          usage() {
            echo "Usage: $0 <host> <port> [timeout]"
            echo "Wait for a TCP port to be available"
            exit 1
          }

          [[ $# -lt 2 ]] && usage

          host="$1"
          port="$2"
          timeout="''${3:-60}"
          elapsed=0

          echo "Waiting for $host:$port to be available (timeout: $timeout seconds)..."

          while ! ${pkgs.netcat-gnu}/bin/nc -z "$host" "$port" 2>/dev/null; do
            if [[ $elapsed -ge $timeout ]]; then
              echo "Timeout waiting for $host:$port"
              exit 1
            fi
            sleep 1
            ((elapsed++))
          done

          echo "$host:$port is available!"
        '';

        # Replacement for Bitnami's ini-file utility using crudini
        iniFile = pkgs.crudini;

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
        # Also handles file collision issues by removing conflicting docs directories
        wheelOverrides = final: prev: {
          # Remove docs directory to avoid file collisions with other packages
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

          # grpcio needs zlib and openssl at runtime
          grpcio = prev.grpcio.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.zlib
              pkgs.openssl
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

          # lxml needs libxml2 and libxslt
          lxml = prev.lxml.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.libxml2
              pkgs.libxslt
            ];
          });

          # mysql-connector-python bundles native libs that need system dependencies
          mysql-connector-python = prev.mysql-connector-python.overrideAttrs (old: {
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.keyutils.lib        # libkeyutils.so.1
              pkgs.libxcrypt-legacy    # libcrypt.so.1 (legacy version)
              pkgs.krb5                # libgssapi_krb5.so.2
            ];
            # Tell auto-patchelf to ignore libs we don't need at runtime
            autoPatchelfIgnoreMissingDeps = [
              "libudev.so.1"  # Only needed for webauthn hardware tokens
            ];
          });
        };

        # 3b. Source build overrides for packages without Linux wheels
        # These packages only have macOS wheels on PyPI, so on Linux they must
        # be built from source. We need to provide setuptools via PYTHONPATH.
        sourceOverrides = final: prev:
          let
            # Get setuptools and cython from nixpkgs
            buildPython = pkgs.python312.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ]);
            # Common preBuild hook to set up Python environment
            setupPythonPath = ''
              export PATH="${buildPython}/bin:$PATH"
              export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
            '';
          in {
          # gssapi needs setuptools + cython + krb5 to build from source
          # Note: krb5 uses split outputs - .lib has libraries, .dev has headers and krb5-config
          gssapi = prev.gssapi.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
              buildPython
              pkgs.krb5.dev    # Headers and krb5-config
              pkgs.pkg-config
            ];
            buildInputs = (old.buildInputs or [ ]) ++ [
              pkgs.krb5.lib    # Libraries
            ];
            preBuild = ''
              ${setupPythonPath}
              # Ensure krb5-config is found and libraries are loadable
              export PATH="${pkgs.krb5.dev}/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
              # Add include path for GSSAPI headers including gssapi_ext.h
              export C_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${C_INCLUDE_PATH:-}"
              export CPLUS_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${CPLUS_INCLUDE_PATH:-}"
            '';
            # Point gssapi to the kerberos libraries (use .lib output)
            GSSAPI_MAIN_LIB = "${pkgs.krb5.lib}/lib/libgssapi_krb5.so";
            GSSAPI_LINKER_ARGS = "-L${pkgs.krb5.lib}/lib";
            # -DHAS_GSSAPI_EXT_H tells gssapi to include gssapi_ext.h which defines
            # gss_key_value_set_desc, GSS_C_NO_CRED_STORE, gss_store_cred_into, etc.
            GSSAPI_COMPILER_ARGS = "-DHAS_GSSAPI_EXT_H -I${pkgs.krb5.dev}/include -I${pkgs.krb5.dev}/include/gssapi";
          });

          # krb5 Python bindings (same requirements as gssapi)
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

          # pykerberos - older package, just needs setuptools + krb5
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

          # python-ldap needs OpenLDAP and SASL libraries
          # Also requires setuptools for distutils (removed in Python 3.12)
          # The pip subprocess doesn't inherit PYTHONPATH, so we need to symlink
          # setuptools directly into the Python's site-packages that pip uses.
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
            # Patch python-ldap's setup.cfg to disable byte_compile which requires distutils
            # The install_lib command handles byte compilation during wheel building
            postPatch = ''
              # Replace compile = 1 with compile = 0 in the [install] section
              # This prevents the byte_compile step that requires distutils
              ${pkgs.gnused}/bin/sed -i 's/compile = 1/compile = 0/g' setup.cfg
              ${pkgs.gnused}/bin/sed -i 's/optimize = 1/optimize = 0/g' setup.cfg
            '';
            preBuild = ''
              export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
              export SETUPTOOLS_USE_DISTUTILS=local
            '';
          });

          # mysqlclient needs MySQL/MariaDB client libraries to build
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

          # python-nvd3 - legacy package with only sdist
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
            pyproject-build-systems.overlays.default  # Build systems FIRST
            overlay                                    # Workspace overlay second
            sourceOverrides                           # Source build overrides third
            wheelOverrides                            # Runtime overrides last
          ]
        );

        # 6. Create the virtual environment with all dependencies
        airflowEnv = pythonSet.mkVirtualEnv "airflow-env" workspace.deps.default;

        # ============================================================
        # Environment Defaults - All Bitnami-compatible defaults
        # ============================================================
        envDefaults = pkgs.writeTextDir "lib/airflow-env-defaults.sh" ''
          #!/usr/bin/env bash
          # Environment variable defaults - matching Bitnami exactly

          # Paths
          export AIRFLOW_HOME="''${AIRFLOW_HOME:-/opt/airflow}"
          export AIRFLOW_DAGS_DIR="''${AIRFLOW_HOME}/dags"
          export AIRFLOW_LOGS_DIR="''${AIRFLOW_HOME}/logs"
          export AIRFLOW_SCHEDULER_LOGS_DIR="''${AIRFLOW_LOGS_DIR}/scheduler"
          export AIRFLOW_PLUGINS_DIR="''${AIRFLOW_HOME}/plugins"
          export AIRFLOW_TMP_DIR="''${AIRFLOW_HOME}/tmp"
          export AIRFLOW_CONF_FILE="''${AIRFLOW_HOME}/airflow.cfg"
          export AIRFLOW_WEBSERVER_CONF_FILE="''${AIRFLOW_HOME}/webserver_config.py"

          # User configuration
          export AIRFLOW_USERNAME="''${AIRFLOW_USERNAME:-user}"
          export AIRFLOW_PASSWORD="''${AIRFLOW_PASSWORD:-bitnami}"
          export AIRFLOW_FIRSTNAME="''${AIRFLOW_FIRSTNAME:-Firstname}"
          export AIRFLOW_LASTNAME="''${AIRFLOW_LASTNAME:-Lastname}"
          export AIRFLOW_EMAIL="''${AIRFLOW_EMAIL:-user@example.com}"

          # Component configuration
          export AIRFLOW_COMPONENT_TYPE="''${AIRFLOW_COMPONENT_TYPE:-api-server}"
          export AIRFLOW_EXECUTOR="''${AIRFLOW_EXECUTOR:-LocalExecutor}"
          export AIRFLOW_SKIP_DB_SETUP="''${AIRFLOW_SKIP_DB_SETUP:-no}"
          export AIRFLOW_LOAD_EXAMPLES="''${AIRFLOW_LOAD_EXAMPLES:-yes}"
          export AIRFLOW_STANDALONE_DAG_PROCESSOR="''${AIRFLOW_STANDALONE_DAG_PROCESSOR:-no}"
          export AIRFLOW_DB_MIGRATE_TIMEOUT="''${AIRFLOW_DB_MIGRATE_TIMEOUT:-120}"
          export AIRFLOW_FORCE_OVERWRITE_CONF_FILE="''${AIRFLOW_FORCE_OVERWRITE_CONF_FILE:-no}"
          export AIRFLOW_HOSTNAME_CALLABLE="''${AIRFLOW_HOSTNAME_CALLABLE:-}"

          # API Server / Webserver configuration
          AIRFLOW_APISERVER_HOST="''${AIRFLOW_APISERVER_HOST:-"''${AIRFLOW_WEBSERVER_HOST:-}"}"
          export AIRFLOW_APISERVER_HOST="''${AIRFLOW_APISERVER_HOST:-127.0.0.1}"
          AIRFLOW_APISERVER_PORT_NUMBER="''${AIRFLOW_APISERVER_PORT_NUMBER:-"''${AIRFLOW_WEBSERVER_PORT_NUMBER:-}"}"
          export AIRFLOW_APISERVER_PORT_NUMBER="''${AIRFLOW_APISERVER_PORT_NUMBER:-8080}"
          AIRFLOW_APISERVER_BASE_URL="''${AIRFLOW_APISERVER_BASE_URL:-"''${AIRFLOW_BASE_URL:-}"}"
          AIRFLOW_APISERVER_BASE_URL="''${AIRFLOW_APISERVER_BASE_URL:-"''${AIRFLOW_WEBSERVER_BASE_URL:-}"}"
          export AIRFLOW_APISERVER_BASE_URL="''${AIRFLOW_APISERVER_BASE_URL:-}"
          export AIRFLOW_WEBSERVER_SECRET_KEY="''${AIRFLOW_WEBSERVER_SECRET_KEY:-airflow-web-server-key}"
          export AIRFLOW_APISERVER_SECRET_KEY="''${AIRFLOW_APISERVER_SECRET_KEY:-airflow-api-server-key}"
          export AIRFLOW_ENABLE_HTTPS="''${AIRFLOW_ENABLE_HTTPS:-no}"
          export AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER="''${AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER:-80}"

          # Triggerer configuration
          export AIRFLOW_TRIGGERER_DEFAULT_CAPACITY="''${AIRFLOW_TRIGGERER_DEFAULT_CAPACITY:-1000}"

          # Worker configuration
          AIRFLOW_WORKER_QUEUE="''${AIRFLOW_WORKER_QUEUE:-"''${AIRFLOW_QUEUE:-}"}"
          export AIRFLOW_WORKER_QUEUE="''${AIRFLOW_WORKER_QUEUE:-}"

          # Pool configuration
          export AIRFLOW_POOL_NAME="''${AIRFLOW_POOL_NAME:-}"
          export AIRFLOW_POOL_SIZE="''${AIRFLOW_POOL_SIZE:-}"
          export AIRFLOW_POOL_DESC="''${AIRFLOW_POOL_DESC:-}"

          # Cryptographic keys
          export AIRFLOW_FERNET_KEY="''${AIRFLOW_FERNET_KEY:-}"
          export AIRFLOW_RAW_FERNET_KEY="''${AIRFLOW_RAW_FERNET_KEY:-}"

          # Database configuration
          export AIRFLOW_DATABASE_HOST="''${AIRFLOW_DATABASE_HOST:-postgresql}"
          export AIRFLOW_DATABASE_PORT_NUMBER="''${AIRFLOW_DATABASE_PORT_NUMBER:-5432}"
          export AIRFLOW_DATABASE_NAME="''${AIRFLOW_DATABASE_NAME:-bitnami_airflow}"
          export AIRFLOW_DATABASE_USERNAME="''${AIRFLOW_DATABASE_USERNAME:-bn_airflow}"
          export AIRFLOW_DATABASE_PASSWORD="''${AIRFLOW_DATABASE_PASSWORD:-}"
          export AIRFLOW_DATABASE_USE_SSL="''${AIRFLOW_DATABASE_USE_SSL:-no}"

          # Redis (Celery) configuration
          export REDIS_HOST="''${REDIS_HOST:-redis}"
          export REDIS_PORT_NUMBER="''${REDIS_PORT_NUMBER:-6379}"
          export REDIS_USER="''${REDIS_USER:-}"
          export REDIS_PASSWORD="''${REDIS_PASSWORD:-}"
          export REDIS_DATABASE="''${REDIS_DATABASE:-1}"
          export AIRFLOW_REDIS_USE_SSL="''${AIRFLOW_REDIS_USE_SSL:-no}"

          # LDAP configuration
          export AIRFLOW_LDAP_ENABLE="''${AIRFLOW_LDAP_ENABLE:-no}"
          export AIRFLOW_LDAP_URI="''${AIRFLOW_LDAP_URI:-}"
          export AIRFLOW_LDAP_SEARCH="''${AIRFLOW_LDAP_SEARCH:-}"
          export AIRFLOW_LDAP_UID_FIELD="''${AIRFLOW_LDAP_UID_FIELD:-}"
          export AIRFLOW_LDAP_BIND_USER="''${AIRFLOW_LDAP_BIND_USER:-}"
          export AIRFLOW_LDAP_BIND_PASSWORD="''${AIRFLOW_LDAP_BIND_PASSWORD:-}"
          export AIRFLOW_LDAP_USER_REGISTRATION="''${AIRFLOW_LDAP_USER_REGISTRATION:-True}"
          export AIRFLOW_LDAP_USER_REGISTRATION_ROLE="''${AIRFLOW_LDAP_USER_REGISTRATION_ROLE:-}"
          export AIRFLOW_LDAP_ROLES_MAPPING="''${AIRFLOW_LDAP_ROLES_MAPPING:-}"
          export AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN="''${AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN:-True}"
          export AIRFLOW_LDAP_USE_TLS="''${AIRFLOW_LDAP_USE_TLS:-False}"
          export AIRFLOW_LDAP_ALLOW_SELF_SIGNED="''${AIRFLOW_LDAP_ALLOW_SELF_SIGNED:-True}"
          export AIRFLOW_LDAP_TLS_CA_CERTIFICATE="''${AIRFLOW_LDAP_TLS_CA_CERTIFICATE:-}"

          # Python cache
          export PYTHONPYCACHEPREFIX="''${PYTHONPYCACHEPREFIX:-/opt/airflow/tmp/pycache}"

          # Debug mode
          export BITNAMI_DEBUG="''${BITNAMI_DEBUG:-false}"
        '';

        # ============================================================
        # File Variable Loading - Docker secrets support
        # ============================================================
        loadEnvFiles = pkgs.writeTextDir "lib/load-env-files.sh" ''
          #!/usr/bin/env bash
          # Load _FILE variables - compatible with Bitnami pattern

          airflow_env_vars=(
            AIRFLOW_USERNAME
            AIRFLOW_PASSWORD
            AIRFLOW_FIRSTNAME
            AIRFLOW_LASTNAME
            AIRFLOW_EMAIL
            AIRFLOW_COMPONENT_TYPE
            AIRFLOW_EXECUTOR
            AIRFLOW_RAW_FERNET_KEY
            AIRFLOW_FORCE_OVERWRITE_CONF_FILE
            AIRFLOW_FERNET_KEY
            AIRFLOW_WEBSERVER_SECRET_KEY
            AIRFLOW_APISERVER_SECRET_KEY
            AIRFLOW_APISERVER_BASE_URL
            AIRFLOW_APISERVER_HOST
            AIRFLOW_APISERVER_PORT_NUMBER
            AIRFLOW_LOAD_EXAMPLES
            AIRFLOW_HOSTNAME_CALLABLE
            AIRFLOW_POOL_NAME
            AIRFLOW_POOL_SIZE
            AIRFLOW_POOL_DESC
            AIRFLOW_STANDALONE_DAG_PROCESSOR
            AIRFLOW_TRIGGERER_DEFAULT_CAPACITY
            AIRFLOW_WORKER_QUEUE
            AIRFLOW_SKIP_DB_SETUP
            PYTHONPYCACHEPREFIX
            AIRFLOW_DB_MIGRATE_TIMEOUT
            AIRFLOW_ENABLE_HTTPS
            AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER
            AIRFLOW_DATABASE_HOST
            AIRFLOW_DATABASE_PORT_NUMBER
            AIRFLOW_DATABASE_NAME
            AIRFLOW_DATABASE_USERNAME
            AIRFLOW_DATABASE_PASSWORD
            AIRFLOW_DATABASE_USE_SSL
            AIRFLOW_REDIS_USE_SSL
            REDIS_HOST
            REDIS_PORT_NUMBER
            REDIS_USER
            REDIS_PASSWORD
            REDIS_DATABASE
            AIRFLOW_LDAP_ENABLE
            AIRFLOW_LDAP_URI
            AIRFLOW_LDAP_SEARCH
            AIRFLOW_LDAP_UID_FIELD
            AIRFLOW_LDAP_BIND_USER
            AIRFLOW_LDAP_BIND_PASSWORD
            AIRFLOW_LDAP_USER_REGISTRATION
            AIRFLOW_LDAP_USER_REGISTRATION_ROLE
            AIRFLOW_LDAP_ROLES_MAPPING
            AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN
            AIRFLOW_LDAP_USE_TLS
            AIRFLOW_LDAP_ALLOW_SELF_SIGNED
            AIRFLOW_LDAP_TLS_CA_CERTIFICATE
          )

          for env_var in "''${airflow_env_vars[@]}"; do
            file_env_var="''${env_var}_FILE"
            if [[ -n "''${!file_env_var:-}" ]]; then
              if [[ -r "''${!file_env_var:-}" ]]; then
                export "''${env_var}=$(< "''${!file_env_var}")"
                unset "''${file_env_var}"
              else
                warn "Skipping export of ''${env_var}. ''${!file_env_var:-} is not readable."
              fi
            fi
          done
          unset airflow_env_vars
        '';

        # ============================================================
        # Library Functions - Core utilities
        # ============================================================
        libFunctions = pkgs.writeTextDir "lib/functions.sh" ''
          #!/usr/bin/env bash
          # Core library functions - Nix paths are absolute

          # Logging functions
          info() { echo "[INFO] $(${pkgs.coreutils}/bin/date '+%Y-%m-%d %H:%M:%S') $*"; }
          warn() { echo "[WARN] $(${pkgs.coreutils}/bin/date '+%Y-%m-%d %H:%M:%S') $*" >&2; }
          error() { echo "[ERROR] $(${pkgs.coreutils}/bin/date '+%Y-%m-%d %H:%M:%S') $*" >&2; }
          debug() { [[ "''${BITNAMI_DEBUG:-false}" == "true" ]] && echo "[DEBUG] $*" >&2; }

          # Boolean checks
          is_boolean_yes() {
            local -r value="''${1:-}"
            case "''${value,,}" in
              yes|true|1) return 0 ;;
              *) return 1 ;;
            esac
          }

          is_empty_value() {
            [[ -z "''${1:-}" ]]
          }

          # File system functions
          ensure_dir_exists() {
            local dir="$1"
            [[ -d "$dir" ]] || ${pkgs.coreutils}/bin/mkdir -p "$dir"
          }

          configure_permissions_ownership() {
            local -r path="''${1:?path is required}"
            local -r user="''${2:-}"
            local -r group="''${3:-}"
            local -r mode="''${4:-}"

            [[ -n "$mode" ]] && ${pkgs.coreutils}/bin/chmod "$mode" "$path"
            [[ -n "$user" ]] && ${pkgs.coreutils}/bin/chown "$user" "$path"
            [[ -n "$group" ]] && ${pkgs.coreutils}/bin/chgrp "$group" "$path"
          }

          # Network functions
          wait_for_port() {
            local host="$1"
            local port="$2"
            local timeout="''${3:-60}"
            ${waitForPort}/bin/wait-for-port "$host" "$port" "$timeout"
          }

          # URL encoding for connection strings
          urlencode() {
            local old_lc_collate="''${LC_COLLATE:-}"
            LC_COLLATE=C
            local length="''${#1}"
            local i c
            for ((i = 0; i < length; i++)); do
              c="''${1:$i:1}"
              case $c in
                [a-zA-Z0-9.~_-]) printf '%s' "$c" ;;
                *) printf '%%%02X' "'$c" ;;
              esac
            done
            LC_COLLATE="$old_lc_collate"
          }

          # Airflow-specific URL encoding (doubles % for config file)
          airflow_encode_url() {
            local url_encoded
            url_encoded=$(urlencode "$1")
            echo "''${url_encoded//\%/\%\%}"
          }

          # Retry with timeout
          retry_while() {
            local -r cmd="''${1:?cmd is missing}"
            local -r retries="''${2:-30}"
            local -r sleep_time="''${3:-5}"
            local attempt=0

            while ! eval "$cmd"; do
              ((attempt++))
              if [[ $attempt -ge $retries ]]; then
                return 1
              fi
              ${pkgs.coreutils}/bin/sleep "$sleep_time"
            done
          }

          # INI file manipulation using crudini
          airflow_conf_set() {
            local section="$1"
            local key="$2"
            local value="$3"
            local file="''${4:-$AIRFLOW_CONF_FILE}"
            ${pkgs.crudini}/bin/crudini --set "$file" "$section" "$key" "$value"
          }

          airflow_conf_get() {
            local section="$1"
            local key="$2"
            local file="''${3:-$AIRFLOW_CONF_FILE}"
            ${pkgs.crudini}/bin/crudini --get "$file" "$section" "$key" 2>/dev/null || echo ""
          }

          # Python config file manipulation (webserver_config.py)
          airflow_webserver_conf_set() {
            local key="$1"
            local value="$2"
            local is_literal="''${3:-no}"
            local file="$AIRFLOW_WEBSERVER_CONF_FILE"

            if is_boolean_yes "$is_literal"; then
              value="'$value'"
            fi

            if ${pkgs.gnugrep}/bin/grep -E -q "^#*[[:space:]]*''${key} =.*$" "$file" 2>/dev/null; then
              ${pkgs.gnused}/bin/sed -E -i "s|^#*[[:space:]]*''${key} =.*$|''${key} = ''${value}|" "$file"
            else
              printf '\n%s = %s' "$key" "$value" >> "$file"
            fi
          }

          # Fernet key processing
          process_fernet_key() {
            if [[ -n "$AIRFLOW_RAW_FERNET_KEY" && -z "$AIRFLOW_FERNET_KEY" ]]; then
              if [[ ''${#AIRFLOW_RAW_FERNET_KEY} -lt 32 ]]; then
                error "AIRFLOW_RAW_FERNET_KEY must have at least 32 characters"
                return 1
              fi
              [[ ''${#AIRFLOW_RAW_FERNET_KEY} -gt 32 ]] && warn "AIRFLOW_RAW_FERNET_KEY truncated to 32 characters"
              AIRFLOW_FERNET_KEY="$(echo -n "''${AIRFLOW_RAW_FERNET_KEY:0:32}" | ${pkgs.coreutils}/bin/base64)"
              export AIRFLOW_FERNET_KEY
            fi
          }

          # Secret key processing
          process_secret_key() {
            local key_var="$1"
            local key_value="''${!key_var}"

            if [[ ''${#key_value} -gt 32 ]]; then
              warn "''${key_var} truncated to 32 characters"
            fi
            printf -v "$key_var" '%s' "$(echo -n "''${key_value:0:32}" | ${pkgs.coreutils}/bin/base64)"
            export "''${key_var}"
          }

          # Get Airflow major version
          airflow_major_version() {
            local raw_version
            raw_version="$(airflow version 2>/dev/null | ${pkgs.gnugrep}/bin/grep -E -v 'WARNING|DEBUG' || echo "3.0.0")"
            echo "''${raw_version%%.*}"
          }

          # Validation functions
          validate_port() {
            local port="$1"
            if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 ]] || [[ "$port" -gt 65535 ]]; then
              error "Invalid port number: $port"
              return 1
            fi
          }

          # File operations
          replace_in_file() {
            local file="$1"
            local search="$2"
            local replace="$3"
            ${pkgs.gnused}/bin/sed -i "s|$search|$replace|g" "$file"
          }
        '';

        # ============================================================
        # Validation Script - Configuration validation
        # ============================================================
        airflowValidate = pkgs.writeShellScriptBin "airflow-validate" ''
          #!${pkgs.bashInteractive}/bin/bash
          set -euo pipefail

          source ${libFunctions}/lib/functions.sh

          error_code=0

          print_validation_error() {
            error "$1"
            error_code=1
          }

          info "Validating Airflow configuration..."

          # Component type validation
          case "$AIRFLOW_COMPONENT_TYPE" in
            api-server|dag-processor|scheduler|triggerer|webserver|worker) ;;
            *)
              print_validation_error "Invalid AIRFLOW_COMPONENT_TYPE: $AIRFLOW_COMPONENT_TYPE"
              print_validation_error "Allowed values: api-server dag-processor scheduler triggerer webserver worker"
              ;;
          esac

          # Executor validation
          if is_empty_value "$AIRFLOW_EXECUTOR"; then
            print_validation_error "AIRFLOW_EXECUTOR is required"
          else
            case "$AIRFLOW_EXECUTOR" in
              LocalExecutor|CeleryExecutor|CeleryKubernetesExecutor|KubernetesExecutor) ;;
              *)
                print_validation_error "Invalid AIRFLOW_EXECUTOR: $AIRFLOW_EXECUTOR"
                print_validation_error "Allowed values: LocalExecutor CeleryExecutor CeleryKubernetesExecutor KubernetesExecutor"
                ;;
            esac
          fi

          # Yes/No validation
          for var in AIRFLOW_STANDALONE_DAG_PROCESSOR AIRFLOW_SKIP_DB_SETUP AIRFLOW_LOAD_EXAMPLES AIRFLOW_FORCE_OVERWRITE_CONF_FILE; do
            val="''${!var:-}"
            if [[ -n "$val" ]]; then
              case "''${val,,}" in
                yes|no|true|false|1|0) ;;
                *) print_validation_error "Invalid value for $var: $val (expected yes/no)" ;;
              esac
            fi
          done

          # Process cryptographic keys
          process_fernet_key || error_code=1
          process_secret_key "AIRFLOW_WEBSERVER_SECRET_KEY"
          process_secret_key "AIRFLOW_APISERVER_SECRET_KEY"

          # Database validation
          if is_empty_value "$AIRFLOW_DATABASE_HOST"; then
            print_validation_error "AIRFLOW_DATABASE_HOST is required"
          fi

          # Celery executor requires Redis
          if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
            if is_empty_value "$REDIS_HOST"; then
              print_validation_error "REDIS_HOST is required for $AIRFLOW_EXECUTOR"
            fi
          fi

          # Component-specific validation
          case "$AIRFLOW_COMPONENT_TYPE" in
            webserver|api-server)
              # LDAP validation
              if is_boolean_yes "''${AIRFLOW_LDAP_ENABLE:-no}"; then
                for var in AIRFLOW_LDAP_URI AIRFLOW_LDAP_SEARCH AIRFLOW_LDAP_UID_FIELD \
                           AIRFLOW_LDAP_BIND_USER AIRFLOW_LDAP_BIND_PASSWORD; do
                  if is_empty_value "''${!var:-}"; then
                    print_validation_error "$var is required when LDAP is enabled"
                  fi
                done
              fi
              # Pool validation
              if [[ -n "''${AIRFLOW_POOL_NAME:-}" ]]; then
                if is_empty_value "''${AIRFLOW_POOL_SIZE:-}" || is_empty_value "''${AIRFLOW_POOL_DESC:-}"; then
                  print_validation_error "AIRFLOW_POOL_SIZE and AIRFLOW_POOL_DESC required with AIRFLOW_POOL_NAME"
                fi
              fi
              ;;
            *)
              # Non-webserver components need API server host for Airflow 3
              if is_empty_value "''${AIRFLOW_APISERVER_HOST:-}"; then
                warn "AIRFLOW_APISERVER_HOST not set, using default 127.0.0.1"
              fi
              ;;
          esac

          if [[ $error_code -eq 0 ]]; then
            info "Configuration validation passed"
          else
            error "Configuration validation failed"
          fi

          exit $error_code
        '';

        # ============================================================
        # Configuration Generation - airflow.cfg and webserver_config.py
        # ============================================================
        airflowConfig = pkgs.writeShellScriptBin "airflow-config" ''
          #!${pkgs.bashInteractive}/bin/bash
          set -euo pipefail

          source ${libFunctions}/lib/functions.sh

          info "Generating Airflow configuration..."

          major_version=$(airflow_major_version)
          debug "Detected Airflow major version: $major_version"

          # Generate base configuration
          case "$AIRFLOW_COMPONENT_TYPE" in
            webserver|api-server)
              if [[ ! -f "$AIRFLOW_CONF_FILE" ]]; then
                info "Generating default airflow.cfg..."
                airflow config list --defaults > "$AIRFLOW_CONF_FILE"
              fi
              if [[ ! -f "$AIRFLOW_WEBSERVER_CONF_FILE" ]]; then
                info "Generating default webserver_config.py..."
                cat > "$AIRFLOW_WEBSERVER_CONF_FILE" << 'PYEOF'
# Airflow webserver configuration
from flask_appbuilder.security.manager import AUTH_DB
# from flask_appbuilder.security.manager import AUTH_LDAP

AUTH_TYPE = AUTH_DB
PYEOF
              fi
              ;;
            *)
              if [[ ! -f "$AIRFLOW_CONF_FILE" ]]; then
                info "Generating default airflow.cfg..."
                airflow config list --defaults > "$AIRFLOW_CONF_FILE"
              fi
              # Non-webserver components need API server configuration for Airflow 3
              if [[ $major_version -ne 2 ]]; then
                airflow_conf_set "api" "host" "$AIRFLOW_APISERVER_HOST"
                airflow_conf_set "core" "execution_api_server_url" \
                  "http://''${AIRFLOW_APISERVER_HOST}:''${AIRFLOW_APISERVER_PORT_NUMBER}/execution/"
              fi
              ;;
          esac

          # Configure database connection
          db_user=$(airflow_encode_url "$AIRFLOW_DATABASE_USERNAME")
          db_pass=$(airflow_encode_url "$AIRFLOW_DATABASE_PASSWORD")
          db_ssl_opt=""
          is_boolean_yes "$AIRFLOW_DATABASE_USE_SSL" && db_ssl_opt="?sslmode=require"

          airflow_conf_set "database" "sql_alchemy_conn" \
            "postgresql+psycopg2://''${db_user}:''${db_pass}@''${AIRFLOW_DATABASE_HOST}:''${AIRFLOW_DATABASE_PORT_NUMBER}/''${AIRFLOW_DATABASE_NAME}''${db_ssl_opt}"

          # Configure port
          if [[ $major_version -eq 2 ]]; then
            airflow_conf_set "webserver" "web_server_port" "$AIRFLOW_APISERVER_PORT_NUMBER"
          else
            airflow_conf_set "api" "port" "$AIRFLOW_APISERVER_PORT_NUMBER"
          fi

          # Configure base URL
          scheme="http"
          is_boolean_yes "$AIRFLOW_ENABLE_HTTPS" && scheme="https"
          base_url="''${scheme}://''${AIRFLOW_APISERVER_HOST}"
          [[ "$AIRFLOW_APISERVER_PORT_NUMBER" != "80" && "$AIRFLOW_APISERVER_PORT_NUMBER" != "443" ]] && \
            base_url="''${base_url}:''${AIRFLOW_APISERVER_PORT_NUMBER}"

          airflow_conf_set "webserver" "base_url" "$base_url"
          [[ $major_version -eq 3 ]] && airflow_conf_set "api" "base_url" "$base_url"

          # Configure secret keys
          [[ -n "$AIRFLOW_FERNET_KEY" ]] && airflow_conf_set "core" "fernet_key" "$AIRFLOW_FERNET_KEY"

          secret_section="api"
          [[ $major_version -eq 2 ]] && secret_section="webserver"
          airflow_conf_set "$secret_section" "secret_key" "$AIRFLOW_WEBSERVER_SECRET_KEY"
          [[ $major_version -ne 2 ]] && airflow_conf_set "api_auth" "jwt_secret" "$AIRFLOW_APISERVER_SECRET_KEY"

          # Configure executor
          airflow_conf_set "core" "executor" "$AIRFLOW_EXECUTOR"
          airflow_conf_set "core" "auth_manager" "airflow.providers.fab.auth_manager.fab_auth_manager.FabAuthManager"

          # Configure Celery if needed
          if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
            info "Configuring Celery executor..."
            redis_user=$(airflow_encode_url "$REDIS_USER")
            redis_pass=$(airflow_encode_url "$REDIS_PASSWORD")
            redis_proto="redis"
            is_boolean_yes "$AIRFLOW_REDIS_USE_SSL" && redis_proto="rediss"

            # Build broker URL
            if [[ -n "$REDIS_USER" ]]; then
              broker_url="''${redis_proto}://''${redis_user}:''${redis_pass}@''${REDIS_HOST}:''${REDIS_PORT_NUMBER}/''${REDIS_DATABASE}"
            else
              broker_url="''${redis_proto}://:''${redis_pass}@''${REDIS_HOST}:''${REDIS_PORT_NUMBER}/''${REDIS_DATABASE}"
            fi
            airflow_conf_set "celery" "broker_url" "$broker_url"

            is_boolean_yes "$AIRFLOW_REDIS_USE_SSL" && airflow_conf_set "celery" "redis_backend_use_ssl" "true"

            airflow_conf_set "celery" "result_backend" \
              "db+postgresql://''${db_user}:''${db_pass}@''${AIRFLOW_DATABASE_HOST}:''${AIRFLOW_DATABASE_PORT_NUMBER}/''${AIRFLOW_DATABASE_NAME}''${db_ssl_opt}"
          fi

          # Configure examples and DAG processor
          if [[ "$AIRFLOW_COMPONENT_TYPE" != "worker" ]]; then
            load_examples="False"
            is_boolean_yes "$AIRFLOW_LOAD_EXAMPLES" && load_examples="True"
            airflow_conf_set "core" "load_examples" "$load_examples"

            standalone_dag="False"
            is_boolean_yes "$AIRFLOW_STANDALONE_DAG_PROCESSOR" && standalone_dag="True"
            airflow_conf_set "scheduler" "standalone_dag_processor" "$standalone_dag"
          fi

          # Configure triggerer capacity
          if [[ "$AIRFLOW_COMPONENT_TYPE" == "triggerer" && -n "$AIRFLOW_TRIGGERER_DEFAULT_CAPACITY" ]]; then
            capacity_key="capacity"
            [[ $major_version -eq 2 ]] && capacity_key="default_capacity"
            airflow_conf_set "triggerer" "$capacity_key" "$AIRFLOW_TRIGGERER_DEFAULT_CAPACITY"
          fi

          # Configure hostname callable if set
          if [[ -n "''${AIRFLOW_HOSTNAME_CALLABLE:-}" ]]; then
            airflow_conf_set "core" "hostname_callable" "$AIRFLOW_HOSTNAME_CALLABLE"
          fi

          # Configure LDAP for webserver/api-server
          if [[ "$AIRFLOW_COMPONENT_TYPE" == "webserver" || "$AIRFLOW_COMPONENT_TYPE" == "api-server" ]]; then
            airflow_conf_set "webserver" "rbac" "true"

            if is_boolean_yes "''${AIRFLOW_LDAP_ENABLE:-no}"; then
              info "Configuring LDAP authentication..."
              ${pkgs.gnused}/bin/sed -i 's/AUTH_TYPE = AUTH_DB/AUTH_TYPE = AUTH_LDAP/' "$AIRFLOW_WEBSERVER_CONF_FILE"
              ${pkgs.gnused}/bin/sed -i 's/from flask_appbuilder.security.manager import AUTH_DB/from flask_appbuilder.security.manager import AUTH_LDAP/' "$AIRFLOW_WEBSERVER_CONF_FILE"

              airflow_webserver_conf_set "AUTH_LDAP_SERVER" "$AIRFLOW_LDAP_URI" "yes"
              airflow_webserver_conf_set "AUTH_LDAP_SEARCH" "$AIRFLOW_LDAP_SEARCH" "yes"
              airflow_webserver_conf_set "AUTH_LDAP_UID_FIELD" "$AIRFLOW_LDAP_UID_FIELD" "yes"
              airflow_webserver_conf_set "AUTH_LDAP_BIND_USER" "$AIRFLOW_LDAP_BIND_USER" "yes"
              airflow_webserver_conf_set "AUTH_LDAP_BIND_PASSWORD" "$AIRFLOW_LDAP_BIND_PASSWORD" "yes"
              airflow_webserver_conf_set "AUTH_USER_REGISTRATION" "$AIRFLOW_LDAP_USER_REGISTRATION" "no"
              [[ -n "$AIRFLOW_LDAP_USER_REGISTRATION_ROLE" ]] && \
                airflow_webserver_conf_set "AUTH_USER_REGISTRATION_ROLE" "$AIRFLOW_LDAP_USER_REGISTRATION_ROLE" "yes"
              [[ -n "$AIRFLOW_LDAP_ROLES_MAPPING" ]] && \
                airflow_webserver_conf_set "AUTH_ROLES_MAPPING" "$AIRFLOW_LDAP_ROLES_MAPPING" "no"
              airflow_webserver_conf_set "AUTH_ROLES_SYNC_AT_LOGIN" "$AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN" "no"
              airflow_webserver_conf_set "AUTH_LDAP_ALLOW_SELF_SIGNED" "$AIRFLOW_LDAP_ALLOW_SELF_SIGNED" "no"

              if [[ "$AIRFLOW_LDAP_USE_TLS" == "True" && -n "$AIRFLOW_LDAP_TLS_CA_CERTIFICATE" ]]; then
                airflow_webserver_conf_set "AUTH_LDAP_TLS_CACERTFILE" "$AIRFLOW_LDAP_TLS_CA_CERTIFICATE" "yes"
              fi
            fi

            # Configure hashing for Airflow 3
            if [[ $major_version -eq 3 ]]; then
              airflow_webserver_conf_set "FAB_PASSWORD_HASH_METHOD" "pbkdf2:sha256" "yes"
            fi
          fi

          # Debug logging
          is_boolean_yes "''${BITNAMI_DEBUG:-false}" && airflow_conf_set "logging" "logging_level" "DEBUG"

          info "Configuration generation complete"
        '';

        # ============================================================
        # Initialization Script - Database setup, user creation
        # ============================================================
        airflowInit = pkgs.writeShellScriptBin "airflow-init" ''
          #!${pkgs.bashInteractive}/bin/bash
          set -euo pipefail

          source ${libFunctions}/lib/functions.sh

          info "Initializing Airflow (component: $AIRFLOW_COMPONENT_TYPE)..."

          # Create directories with proper permissions
          for dir in "$AIRFLOW_HOME" "$AIRFLOW_DAGS_DIR" "$AIRFLOW_LOGS_DIR" \
                     "$AIRFLOW_SCHEDULER_LOGS_DIR" "$AIRFLOW_PLUGINS_DIR" "$AIRFLOW_TMP_DIR"; do
            ensure_dir_exists "$dir"
          done

          # Generate configuration if needed
          if [[ ! -f "$AIRFLOW_CONF_FILE" ]] || is_boolean_yes "''${AIRFLOW_FORCE_OVERWRITE_CONF_FILE:-no}"; then
            airflow-config
          else
            info "Using existing configuration file"
          fi

          # Wait for database
          info "Waiting for database at ''${AIRFLOW_DATABASE_HOST}:''${AIRFLOW_DATABASE_PORT_NUMBER}..."
          wait_for_port "$AIRFLOW_DATABASE_HOST" "$AIRFLOW_DATABASE_PORT_NUMBER" 120

          # Database operations
          major_version=$(airflow_major_version)
          db_init_cmd="migrate"
          [[ $major_version -eq 2 ]] && db_init_cmd="init"

          case "$AIRFLOW_COMPONENT_TYPE" in
            webserver|api-server)
              # Remove stale PID file
              rm -f "''${AIRFLOW_TMP_DIR}/airflow-''${AIRFLOW_COMPONENT_TYPE}.pid"

              if is_boolean_yes "$AIRFLOW_SKIP_DB_SETUP"; then
                info "Skipping DB setup, waiting for migrations..."
                retry_while "airflow db check-migrations --migration-wait-timeout=$AIRFLOW_DB_MIGRATE_TIMEOUT" 30 10
              elif ! airflow db check-migrations 2>/dev/null; then
                info "Initializing database..."
                airflow db $db_init_cmd

                # Create admin user
                if [[ -n "$AIRFLOW_USERNAME" && -n "$AIRFLOW_PASSWORD" ]]; then
                  info "Creating admin user..."
                  airflow users create \
                    -r "Admin" \
                    -u "$AIRFLOW_USERNAME" \
                    -e "$AIRFLOW_EMAIL" \
                    -p "$AIRFLOW_PASSWORD" \
                    -f "$AIRFLOW_FIRSTNAME" \
                    -l "$AIRFLOW_LASTNAME" || true
                fi

                # Create pool if specified
                if [[ -n "''${AIRFLOW_POOL_NAME:-}" && -n "''${AIRFLOW_POOL_SIZE:-}" && -n "''${AIRFLOW_POOL_DESC:-}" ]]; then
                  info "Creating pool: $AIRFLOW_POOL_NAME"
                  if [[ $major_version -eq 2 ]]; then
                    airflow pool -s "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" || true
                  else
                    airflow pools set "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" || true
                  fi
                fi

                info "Synchronizing permissions..."
                airflow sync-perm --include-dags || true
              else
                info "Database already initialized, running migrations..."
                airflow db migrate

                # Check if admin exists
                if ! airflow users list --output plain 2>/dev/null | ${pkgs.gnugrep}/bin/grep -q "$AIRFLOW_USERNAME"; then
                  info "Creating admin user..."
                  airflow users create \
                    -r "Admin" \
                    -u "$AIRFLOW_USERNAME" \
                    -e "$AIRFLOW_EMAIL" \
                    -p "$AIRFLOW_PASSWORD" \
                    -f "$AIRFLOW_FIRSTNAME" \
                    -l "$AIRFLOW_LASTNAME" || true
                fi

                airflow sync-perm --include-dags || true
              fi
              ;;

            *)
              # Workers, schedulers, triggerers wait for webserver to init
              info "Waiting for database migrations..."
              retry_while "airflow db check-migrations --migration-wait-timeout=$AIRFLOW_DB_MIGRATE_TIMEOUT" 60 10

              info "Waiting for admin user..."
              retry_while "airflow users list --output plain 2>/dev/null | ${pkgs.gnugrep}/bin/grep -q '$AIRFLOW_USERNAME'" 60 5

              # Celery components wait for Redis
              if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
                info "Waiting for Redis at ''${REDIS_HOST}:''${REDIS_PORT_NUMBER}..."
                wait_for_port "$REDIS_HOST" "$REDIS_PORT_NUMBER" 60
              fi
              ;;
          esac

          info "Initialization complete"
        '';

        # ============================================================
        # Main Entrypoint - Orchestrates startup flow
        # ============================================================
        entrypointScript = pkgs.writeScript "entrypoint-unwrapped" ''
          #!${pkgs.bashInteractive}/bin/bash
          set -euo pipefail

          # 1. Load environment defaults
          source ${envDefaults}/lib/airflow-env-defaults.sh

          # 2. Load _FILE variables (Docker secrets)
          source ${loadEnvFiles}/lib/load-env-files.sh

          # 3. Load library functions
          source ${libFunctions}/lib/functions.sh

          info "Firestream Airflow Container Starting"
          info "Component: ''${AIRFLOW_COMPONENT_TYPE:-api-server}"

          # 4. Validate configuration
          if ! airflow-validate; then
            error "Configuration validation failed"
            exit 1
          fi

          # 5. Install custom requirements if present
          if [[ -f "/bitnami/python/requirements.txt" ]]; then
            info "Installing custom Python requirements..."
            pip install --quiet -r /bitnami/python/requirements.txt
          fi

          # 6. Initialize Airflow
          airflow-init

          # 7. Determine command based on component type and Airflow version
          major_version=$(airflow_major_version)

          case "$AIRFLOW_COMPONENT_TYPE" in
            api-server)
              if [[ $major_version -eq 2 ]]; then
                cmd=(airflow webserver)
              else
                cmd=(airflow api-server)
              fi
              ;;
            webserver)
              cmd=(airflow webserver)
              ;;
            scheduler)
              cmd=(airflow scheduler)
              ;;
            triggerer)
              cmd=(airflow triggerer)
              ;;
            dag-processor)
              cmd=(airflow dag-processor)
              ;;
            worker)
              cmd=(airflow celery worker)
              [[ -n "''${AIRFLOW_WORKER_QUEUE:-}" ]] && cmd+=(-q "$AIRFLOW_WORKER_QUEUE")
              ;;
            *)
              error "Unknown component: $AIRFLOW_COMPONENT_TYPE"
              exit 1
              ;;
          esac

          # Add PID file
          cmd+=(--pid "''${AIRFLOW_TMP_DIR}/airflow-''${AIRFLOW_COMPONENT_TYPE}.pid")

          # 8. Execute
          info "Starting: ''${cmd[*]}"
          exec "''${cmd[@]}"
        '';

        # Wrapped entrypoint with full PATH injection (includes airflow, psql, git, etc.)
        entrypoint = pkgs.runCommand "entrypoint" {
          nativeBuildInputs = [ pkgs.makeWrapper ];
        } ''
          mkdir -p $out/bin
          makeWrapper ${entrypointScript} $out/bin/entrypoint \
            --set PATH "${lib.makeBinPath (runtimeBinDeps ++ [ airflowEnv airflowInit airflowValidate airflowConfig ])}" \
            --set PYTHONPATH "${airflowEnv}/${python.sitePackages}" \
            --set LD_LIBRARY_PATH "${lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib ]}" \
            --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
            --set NIX_SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        '';

        # Unified runtime environment - bundles everything for Docker
        runtimeEnv = pkgs.buildEnv {
          name = "airflow-runtime-env";
          paths = runtimeBinDeps ++ [
            airflowEnv
            entrypoint
            airflowInit
            airflowValidate
            airflowConfig
            waitForPort
            iniFile
            envDefaults
            loadEnvFiles
            libFunctions
            pkgs.bashInteractive
            pkgs.cacert
            pkgs.stdenv.cc.cc.lib
          ];
          pathsToLink = [ "/bin" "/lib" "/lib64" "/share" "/etc" ];
        };

        # Docker image definition
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "firestream-airflow";
          tag = "${airflowVersion}-nix";

          contents = pkgs.buildEnv {
            name = "airflow-root";
            paths = systemDeps ++ [
              airflowEnv
              waitForPort
              iniFile
              libFunctions
              airflowInit
              entrypoint
              pkgs.bashInteractive
              pkgs.cacert
              # libstdc++ required by many binary wheels
              pkgs.stdenv.cc.cc.lib
            ];
            pathsToLink = [ "/bin" "/lib" "/lib64" "/share" ];
          };

          config = {
            Env = [
              "PATH=${airflowEnv}/bin:/bin:/usr/bin"
              "AIRFLOW_HOME=/opt/airflow"
              "PYTHONPATH=${airflowEnv}/${python.sitePackages}"
              "LD_LIBRARY_PATH=/lib:/lib64:${pkgs.stdenv.cc.cc.lib}/lib"
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              "NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];

            WorkingDir = "/opt/airflow";

            Entrypoint = [ "${entrypoint}/bin/entrypoint" ];

            ExposedPorts = {
              "8080/tcp" = {};  # Webserver/API
              "8125/tcp" = {};  # StatsD
              "8793/tcp" = {};  # Triggerer
              "8794/tcp" = {};  # Additional services
            };

            User = "1001:1001";

            Volumes = {
              "/opt/airflow/dags" = {};
              "/opt/airflow/logs" = {};
              "/opt/airflow/plugins" = {};
              "/bitnami/python" = {};  # For custom requirements
            };
          };
        };

        # Development shell for testing
        devShell = pkgs.mkShell {
          buildInputs = systemDeps ++ [
            airflowEnv
            waitForPort
            iniFile
            pkgs.uv
            pkgs.docker
            pkgs.docker-compose
          ];

          shellHook = ''
            echo "Firestream Airflow Nix Development Environment"
            echo "Python: ${python}/bin/python"
            echo "Airflow Version: ${airflowVersion}"
            echo ""
            echo "Available commands:"
            echo "  nix build .#dockerImage    - Build the Docker image"
            echo "  nix develop               - Enter development shell"
            echo "  uv sync                   - Sync Python dependencies"
            echo ""
          '';
        };

      in {
        # Packages available on all systems
        packages = {
          inherit airflowEnv waitForPort iniFile entrypoint airflowInit airflowValidate airflowConfig envDefaults loadEnvFiles runtimeEnv;
        } // lib.optionalAttrs isLinux {
          # Docker images only available on Linux
          default = dockerImage;
          inherit dockerImage;
        };

        devShells.default = devShell;

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
      });
}