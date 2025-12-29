# PostgreSQL Container Module - Using Firestream Factories
# Copyright Firestream. Apache-2.0 License.
#
# This module defines the PostgreSQL container using mkContainerModule.
# The factory handles:
# - Entrypoint generation
# - Docker image building
# - Environment loading and secrets
# - Development shell
#
# Usage:
#   postgresqlModule = import ./module.nix {
#     inherit pkgs lib firestream;
#     version = "17";  # or "16"
#   };

{ pkgs
, lib
, firestream
, version ? "17"
}:

let
  # Select PostgreSQL version
  postgresql = pkgs."postgresql_${version}";

  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;

  # PostgreSQL-specific helper functions (needed by scripts)
  # These supplement the core library functions
  postgresqlHelpers = ''
    ########################
    # Enable NSS wrapper for arbitrary UID support
    # Arguments:
    #   None
    # Returns:
    #   None
    #########################
    postgresql_enable_nss_wrapper() {
      if ! getent passwd "$(id -u)" &>/dev/null && [ -e "$NSS_WRAPPER_LIB" ]; then
        debug "Configuring libnss_wrapper..."
        export LD_PRELOAD="$NSS_WRAPPER_LIB"
        export NSS_WRAPPER_PASSWD="$(mktemp)"
        export NSS_WRAPPER_GROUP="$(mktemp)"
        echo "postgres:x:$(id -u):$(id -g):PostgreSQL:$POSTGRESQL_DATA_DIR:/bin/false" >"$NSS_WRAPPER_PASSWD"
        echo "postgres:x:$(id -g):" >"$NSS_WRAPPER_GROUP"
      fi
    }

    ########################
    # Print validation error
    # Arguments:
    #   $1 - error message
    # Returns:
    #   Sets error_code=1
    #########################
    print_validation_error() {
      error "$1"
      error_code=1
    }

    ########################
    # Replace text in file
    # Arguments:
    #   $1 - file
    #   $2 - search pattern
    #   $3 - replacement
    #########################
    replace_in_file() {
      local file="$1"
      local search="$2"
      local replace="$3"
      ${pkgs.gnused}/bin/sed -i "s|$search|$replace|g" "$file"
    }

    ########################
    # Ensure directory exists
    # Arguments:
    #   $1 - directory path
    #########################
    ensure_dir_exists() {
      local dir="$1"
      [[ -d "$dir" ]] || mkdir -p "$dir"
    }

    ########################
    # Check if value is boolean yes
    # Arguments:
    #   $1 - value
    # Returns:
    #   0 if yes/true/1, 1 otherwise
    #########################
    is_boolean_yes() {
      local bool="''${1:-}"
      [[ "$bool" =~ ^(yes|true|1)$ ]]
    }

    ########################
    # Check if value is empty
    # Arguments:
    #   $1 - value
    # Returns:
    #   0 if empty, 1 otherwise
    #########################
    is_empty_value() {
      local value="''${1:-}"
      [[ -z "$value" ]]
    }

    ########################
    # Validate port number
    # Arguments:
    #   $1 - port
    # Returns:
    #   0 if valid, 1 otherwise
    #########################
    validate_port() {
      local port="$1"
      [[ "$port" =~ ^[0-9]+$ ]] && [[ "$port" -ge 1 ]] && [[ "$port" -le 65535 ]]
    }

    ########################
    # Wait for TCP port (wrapper around wait-for-port binary)
    # Arguments:
    #   $1 - host
    #   $2 - port
    #   $3 - timeout (optional, default 60)
    #########################
    wait_for_port() {
      local host="$1"
      local port="$2"
      local timeout="''${3:-60}"

      info "Waiting for $host:$port to be available (timeout: $timeout seconds)..."
      if wait-for-port --host "$host" --timeout "$timeout" "$port"; then
        info "$host:$port is available!"
      else
        error "Timeout waiting for $host:$port"
        return 1
      fi
    }
  '';

  # System dependencies (libs needed in the container)
  systemDeps = with pkgs; [
    # SSL/TLS
    cacert openssl

    # LDAP support
    openldap cyrus_sasl

    # Compression
    bzip2 xz zlib zstd lz4

    # XML support
    libxml2

    # Utilities
    coreutils gnugrep gnused gawk findutils which curl netcat-gnu gzip
    procps util-linux

    # Terminal
    ncurses readline

    # NSS wrapper for random UID support
    nss_wrapper
  ];

  # Runtime binary deps (need to be in PATH)
  runtimeBinDeps = with pkgs; [
    coreutils bash gnused gnugrep gawk findutils which
    postgresql gzip netcat-gnu
  ];

in firestream.mkContainerModule {
  name = "postgresql";
  inherit version;

  # Paths configuration (Bitnami compatibility)
  paths = {
    base = "/opt/bitnami/postgresql";
    conf = "/opt/bitnami/postgresql/conf";
    data = "/bitnami/postgresql/data";
    logs = "/opt/bitnami/postgresql/logs";
  };

  # Environment variables with defaults
  envVars = {
    # Base directories
    POSTGRESQL_BASE_DIR = "/opt/bitnami/postgresql";
    POSTGRESQL_VOLUME_DIR = "/bitnami/postgresql";
    POSTGRESQL_DATA_DIR = "\${POSTGRESQL_VOLUME_DIR}/data";
    POSTGRESQL_CONF_DIR = "\${POSTGRESQL_BASE_DIR}/conf";
    POSTGRESQL_MOUNTED_CONF_DIR = "\${POSTGRESQL_VOLUME_DIR}/conf";
    POSTGRESQL_DEFAULT_CONF_DIR = "\${POSTGRESQL_BASE_DIR}/conf.default";
    POSTGRESQL_CONF_FILE = "\${POSTGRESQL_CONF_DIR}/postgresql.conf";
    POSTGRESQL_PGHBA_FILE = "\${POSTGRESQL_CONF_DIR}/pg_hba.conf";
    POSTGRESQL_LOG_DIR = "\${POSTGRESQL_BASE_DIR}/logs";
    POSTGRESQL_LOG_FILE = "\${POSTGRESQL_LOG_DIR}/postgresql.log";
    POSTGRESQL_TMP_DIR = "\${POSTGRESQL_BASE_DIR}/tmp";
    POSTGRESQL_PID_FILE = "\${POSTGRESQL_TMP_DIR}/postgresql.pid";
    POSTGRESQL_INITSCRIPTS_DIR = "/docker-entrypoint-initdb.d";
    POSTGRESQL_PREINITSCRIPTS_DIR = "/docker-entrypoint-preinitdb.d";

    # User and group
    POSTGRESQL_DAEMON_USER = "postgres";
    POSTGRESQL_DAEMON_GROUP = "postgres";

    # Connection settings
    POSTGRESQL_PORT_NUMBER = "5432";
    POSTGRESQL_ALLOW_REMOTE_CONNECTIONS = "yes";
    POSTGRESQL_INIT_MAX_TIMEOUT = "60";

    # Authentication
    POSTGRESQL_PASSWORD = "";
    POSTGRESQL_USERNAME = "postgres";
    POSTGRESQL_DATABASE = "";
    POSTGRESQL_POSTGRES_PASSWORD = "";
    ALLOW_EMPTY_PASSWORD = "no";

    # Replication
    POSTGRESQL_REPLICATION_MODE = "";
    POSTGRESQL_REPLICATION_USER = "";
    POSTGRESQL_REPLICATION_PASSWORD = "";
    POSTGRESQL_MASTER_HOST = "";
    POSTGRESQL_MASTER_PORT_NUMBER = "5432";
    POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS = "0";
    POSTGRESQL_SYNCHRONOUS_COMMIT_MODE = "on";
    POSTGRESQL_CLUSTER_APP_NAME = "walreceiver";
    POSTGRESQL_WAL_LEVEL = "replica";
    POSTGRESQL_REPLICATION_USE_PASSFILE = "no";
    POSTGRESQL_REPLICATION_PASSFILE_PATH = "\${POSTGRESQL_CONF_DIR}/.pgpass";

    # TLS
    POSTGRESQL_ENABLE_TLS = "no";
    POSTGRESQL_TLS_CERT_FILE = "";
    POSTGRESQL_TLS_KEY_FILE = "";
    POSTGRESQL_TLS_CA_FILE = "";
    POSTGRESQL_TLS_CRL_FILE = "";
    POSTGRESQL_TLS_PREFER_SERVER_CIPHERS = "yes";

    # LDAP
    POSTGRESQL_ENABLE_LDAP = "no";
    POSTGRESQL_LDAP_URL = "";
    POSTGRESQL_LDAP_SERVER = "";
    POSTGRESQL_LDAP_PORT = "";
    POSTGRESQL_LDAP_PREFIX = "";
    POSTGRESQL_LDAP_SUFFIX = "";
    POSTGRESQL_LDAP_BASE_DN = "";
    POSTGRESQL_LDAP_BIND_DN = "";
    POSTGRESQL_LDAP_BIND_PASSWORD = "";
    POSTGRESQL_LDAP_SEARCH_ATTR = "";
    POSTGRESQL_LDAP_SEARCH_FILTER = "";
    POSTGRESQL_LDAP_TLS = "";
    POSTGRESQL_LDAP_SCHEME = "";

    # Other settings
    POSTGRESQL_SHUTDOWN_MODE = "fast";
    POSTGRESQL_PGCTLTIMEOUT = "60";
    POSTGRESQL_FIRST_BOOT = "yes";
    POSTGRESQL_INITDB_ARGS = "";
    POSTGRESQL_INITDB_WAL_DIR = "";
    POSTGRESQL_SHARED_PRELOAD_LIBRARIES = "";
    POSTGRESQL_FSYNC = "on";
    POSTGRESQL_DEFAULT_TOAST_COMPRESSION = "";
    POSTGRESQL_PASSWORD_ENCRYPTION = "";

    # NSS wrapper
    NSS_WRAPPER_LIB = "${pkgs.nss_wrapper}/lib/libnss_wrapper.so";

    # Debug mode
    BITNAMI_DEBUG = "false";
  };

  # Variables supporting _FILE suffix for Docker secrets
  envVarsWithSecrets = [
    "POSTGRESQL_PASSWORD"
    "POSTGRESQL_POSTGRES_PASSWORD"
    "POSTGRESQL_REPLICATION_PASSWORD"
    "POSTGRESQL_TLS_CERT_FILE"
    "POSTGRESQL_TLS_KEY_FILE"
    "POSTGRESQL_TLS_CA_FILE"
    "POSTGRESQL_TLS_CRL_FILE"
    "POSTGRESQL_LDAP_URL"
    "POSTGRESQL_LDAP_BIND_PASSWORD"
    "POSTGRESQL_INITDB_ARGS"
    "POSTGRESQL_SHARED_PRELOAD_LIBRARIES"
  ];

  # Declarative directory schema
  runtimeDirs = {
    data = {
      path = "/bitnami/postgresql/data";
      type = "data";
      persistence = "persistent";
      mode = "0700";
      owner = 1001;
      group = 1001;
      description = "PostgreSQL data directory";
    };
    conf = {
      path = "/opt/bitnami/postgresql/conf";
      type = "conf";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "PostgreSQL configuration directory";
    };
    confDefault = {
      path = "/opt/bitnami/postgresql/conf.default";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Default configuration templates";
    };
    confD = {
      path = "/opt/bitnami/postgresql/conf/conf.d";
      type = "conf";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Additional configuration directory";
    };
    logs = {
      path = "/opt/bitnami/postgresql/logs";
      type = "logs";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "PostgreSQL log directory";
    };
    tmp = {
      path = "/opt/bitnami/postgresql/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files";
    };
    run = {
      path = "/run/postgresql";
      type = "tmp";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Runtime socket directory";
    };
    volumeDir = {
      path = "/bitnami/postgresql";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Persistent volume root";
    };
    preinitdb = {
      path = "/docker-entrypoint-preinitdb.d";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Pre-initialization scripts";
    };
    initdb = {
      path = "/docker-entrypoint-initdb.d";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Initialization scripts";
    };
  };

  # Validation function
  # Prepend PostgreSQL helpers so functions are available
  validateFn = postgresqlHelpers + ''
    error_code=0
    ${validateScript}
    [[ "$error_code" -eq 0 ]] || exit "$error_code"
  '';

  # Activation: Load secrets and enable NSS wrapper
  activateFn = postgresqlHelpers + ''
    info "Activating PostgreSQL configuration..."

    # Enable NSS wrapper for arbitrary UID support
    postgresql_enable_nss_wrapper

    # Load secrets from _FILE variables
    ${secretsScript}

    info "PostgreSQL configuration activated"
  '';

  # Configuration generation
  configFn = postgresqlHelpers + configScript;

  # Initialization (database setup, users, replication)
  initFn = postgresqlHelpers + initScript;

  # Startup command
  runCmd = ''
    ${postgresqlHelpers}

    # Enable NSS wrapper
    postgresql_enable_nss_wrapper

    info "Starting PostgreSQL..."
    exec postgres \
      -D "$POSTGRESQL_DATA_DIR" \
      --config-file="$POSTGRESQL_CONF_FILE" \
      --hba_file="$POSTGRESQL_PGHBA_FILE"
  '';

  inherit systemDeps runtimeBinDeps;

  exposedPorts = [ 5432 ];
  volumes = [ "/bitnami/postgresql" "/docker-entrypoint-initdb.d" "/docker-entrypoint-preinitdb.d" ];

  user = { name = "postgres"; uid = 1001; gid = 1001; };

  # Development shell extras
  devShellPackages = with pkgs; [ docker docker-compose ];
  devShellHook = ''
    echo "PostgreSQL Version: ${version}"
    echo "PostgreSQL Binary: ${postgresql}/bin/postgres"
    echo ""
    echo "Build commands:"
    echo "  nix build .#dockerImage    - Build the Docker image"
    echo "  docker load < result       - Load image into Docker"
  '';
}
