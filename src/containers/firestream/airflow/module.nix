# Airflow Container Module - Using Firestream Factories
# Copyright Firestream. Apache-2.0 License.
#
# This module defines the Airflow container using mkPythonContainerModule.
# Much simpler than manual implementation - the factory handles:
# - Entrypoint generation
# - Docker image building
# - Environment loading and secrets
# - Development shell
#
# Usage:
#   airflowModule = import ./module.nix {
#     inherit pkgs lib firestream pythonEnv airflowVersion python;
#   };

{ pkgs
, lib
, firestream
, pythonEnv          # The virtual environment from uv2nix
, airflowVersion     # e.g., "3.0.3"
, python ? pkgs.python312
}:

let
  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;

  # Airflow-specific helper functions (needed by scripts)
  # These are adapted from Bitnami's libairflow.sh and libversion.sh
  airflowHelpers = ''
    ########################
    # Get semantic version component
    # Arguments:
    #   $1 - version string (e.g., "3.0.3")
    #   $2 - section: 1=major, 2=minor, 3=patch
    # Returns:
    #   Version component
    #########################
    get_sematic_version() {
        local version="''${1:?version is required}"
        local section="''${2:?section is required}"
        echo "$version" | cut -d. -f"$section"
    }

    ########################
    # Get Airflow major version
    # Arguments:
    #   None
    # Returns:
    #   Airflow major version (e.g., 2 or 3)
    #########################
    airflow_major_version() {
        local raw_version
        raw_version=$(airflow version 2>/dev/null | grep -v "WARNING\|DEBUG" | head -1)
        get_sematic_version "$raw_version" 1
    }

    ########################
    # URL-encode a string for use in connection strings
    # Arguments:
    #   $1 - string to encode
    # Returns:
    #   URL-encoded string
    #########################
    airflow_encode_url() {
        local string="''${1:-}"
        printf '%s' "$string" | python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=str()))"
    }

    ########################
    # Generate Airflow configuration (wrapper for config generation)
    # In the Nix module system, config is generated during activation phase
    # This function is called during init for backwards compatibility
    # Arguments:
    #   None
    # Returns:
    #   None
    #########################
    airflow_configure() {
        debug "airflow_configure called - config generation handled by activation phase"
        # Config file generation is handled by activateFn
        # This function exists for compatibility with init scripts
        if [[ ! -f "$AIRFLOW_CONF_FILE" ]]; then
            warn "Configuration file not found at $AIRFLOW_CONF_FILE - generating defaults"
            airflow config list --defaults > "$AIRFLOW_CONF_FILE"
        fi
    }

    ########################
    # Generate a cryptographically secure secret key
    # Matches Bitnami's airflow_generate_secret_key() pattern
    # Arguments:
    #   $1 - length (default: 32)
    # Returns:
    #   Base64-encoded random string suitable for Airflow secrets
    #########################
    generate_secret_key() {
        local length="''${1:-32}"
        # Generate random bytes, base64 encode, filter to alphanumeric, truncate
        head -c 256 /dev/urandom | base64 | tr -dc 'a-zA-Z0-9' | head -c "$length" | base64
    }

    ########################
    # Process and encode a secret key variable
    # Auto-generates if empty, truncates to 32 chars, base64 encodes
    # Matches Bitnami's secret handling pattern
    # Arguments:
    #   $1 - variable name
    #########################
    process_secret_key() {
        local var_name="''${1:?variable name required}"
        local value="''${!var_name:-}"

        # Auto-generate if empty
        if [[ -z "$value" ]]; then
            value=$(generate_secret_key 32)
            debug "Auto-generated $var_name"
        fi

        # Truncate to 32 chars if needed (Bitnami compatibility)
        if [[ ''${#value} -gt 32 ]]; then
            warn "$var_name has more than 32 characters, truncating"
            value="''${value:0:32}"
        fi

        # Base64 encode for Airflow config (Bitnami encodes secrets)
        export "$var_name"="$(echo -n "$value" | base64)"
        debug "Processed $var_name (base64 encoded)"
    }

    ########################
    # Process Fernet key (special handling)
    # Converts raw key to proper Fernet format
    # Supports AIRFLOW_RAW_FERNET_KEY for user-provided keys
    #########################
    process_fernet_key() {
        local raw="''${AIRFLOW_RAW_FERNET_KEY:-}"

        if [[ -n "$raw" && -z "''${AIRFLOW_FERNET_KEY:-}" ]]; then
            # User provided raw key - validate and convert
            if [[ ''${#raw} -lt 32 ]]; then
                error "AIRFLOW_RAW_FERNET_KEY must have at least 32 characters"
                return 1
            fi
            AIRFLOW_FERNET_KEY="$(echo -n "''${raw:0:32}" | base64)"
            export AIRFLOW_FERNET_KEY
            debug "Converted AIRFLOW_RAW_FERNET_KEY to AIRFLOW_FERNET_KEY"
        elif [[ -z "''${AIRFLOW_FERNET_KEY:-}" ]]; then
            # Auto-generate if not provided
            AIRFLOW_FERNET_KEY="$(generate_secret_key 32)"
            export AIRFLOW_FERNET_KEY
            debug "Auto-generated AIRFLOW_FERNET_KEY"
        fi
    }

    ########################
    # Process all Airflow secret keys
    # Called during validation phase to ensure all secrets are set
    #########################
    process_airflow_secrets() {
        info "Processing Airflow secrets..."

        # Process Fernet key first (special handling)
        process_fernet_key || return 1

        # Process other secret keys
        process_secret_key "AIRFLOW_WEBSERVER_SECRET_KEY"
        process_secret_key "AIRFLOW_APISERVER_SECRET_KEY"
        process_secret_key "AIRFLOW_JWT_SECRET_KEY"

        info "All secrets processed successfully"
    }
  '';

  # System dependencies (libs needed in the container)
  systemDeps = with pkgs; [
    # SSL/TLS and crypto
    cacert openssl krb5
    # LDAP
    openldap cyrus_sasl
    # Database clients
    postgresql mariadb-connector-c
    # Compression
    bzip2 xz zlib zstd
    # Version control
    git openssh
    # Process management
    procps
    # Utilities
    curl wget netcat-gnu
    # Terminal
    ncurses readline
    # INI file manipulation
    crudini jq
    # C++ runtime
    stdenv.cc.cc.lib
  ];

  # Runtime binary deps (need to be in PATH)
  runtimeBinDeps = with pkgs; [
    coreutils bash gnused gnugrep gawk findutils which
    postgresql git crudini curl netcat-gnu openssh jq
    firestream.waitForPortPkg  # Required by init.sh for database/redis readiness checks
  ];

  # Build-time: Airflow config template with {{PLACEHOLDER}} syntax
  # Runtime activation replaces these with actual environment values
  airflowConfigTemplate = ''
    # Airflow configuration template
    # Placeholders are replaced at container startup via activateFn

    [core]
    executor = {{AIRFLOW_EXECUTOR}}
    dags_folder = /opt/airflow/dags
    plugins_folder = /opt/airflow/plugins
    load_examples = {{AIRFLOW_LOAD_EXAMPLES}}
    fernet_key = {{AIRFLOW_FERNET_KEY}}
    auth_manager = airflow.providers.fab.auth_manager.fab_auth_manager.FabAuthManager
    execution_api_server_url = {{AIRFLOW_EXECUTION_API_SERVER_URL}}

    [database]
    sql_alchemy_conn = {{AIRFLOW_DATABASE_SQL_ALCHEMY_CONN}}

    [webserver]
    web_server_port = {{AIRFLOW_APISERVER_PORT_NUMBER}}
    base_url = {{AIRFLOW_BASE_URL}}
    secret_key = {{AIRFLOW_WEBSERVER_SECRET_KEY}}
    rbac = true

    [api]
    port = {{AIRFLOW_APISERVER_PORT_NUMBER}}
    host = {{AIRFLOW_APISERVER_HOST}}
    secret_key = {{AIRFLOW_APISERVER_SECRET_KEY}}

    [api_auth]
    jwt_secret = {{AIRFLOW_JWT_SECRET_KEY}}

    [scheduler]
    dag_dir_list_interval = 300
    standalone_dag_processor = {{AIRFLOW_STANDALONE_DAG_PROCESSOR}}

    [celery]
    broker_url = {{AIRFLOW_CELERY_BROKER_URL}}
    result_backend = {{AIRFLOW_CELERY_RESULT_BACKEND}}

    [triggerer]
    capacity = {{AIRFLOW_TRIGGERER_DEFAULT_CAPACITY}}

    [logging]
    base_log_folder = /opt/airflow/logs
    logging_level = {{AIRFLOW_LOGGING_LEVEL}}

    [fab]
    config_file = /opt/airflow/webserver_config.py
  '';

  # Default webserver_config.py template
  webserverConfigTemplate = ''
    # Airflow webserver configuration
    # Generated by Firestream Nix module
    from flask_appbuilder.security.manager import AUTH_DB
    # from flask_appbuilder.security.manager import AUTH_LDAP

    AUTH_TYPE = AUTH_DB
    FAB_PASSWORD_HASH_METHOD = 'pbkdf2:sha256'
  '';

in firestream.mkPythonContainerModule {
  name = "airflow";
  version = airflowVersion;
  inherit pythonEnv python;

  # Paths configuration (Airflow uses /opt/airflow structure)
  paths = {
    base = "/opt/airflow";
    conf = "/opt/airflow";
    data = "/opt/airflow/dags";
    logs = "/opt/airflow/logs";
  };

  # Environment variables with defaults (from env-defaults.sh)
  envVars = {
    # Paths
    AIRFLOW_HOME = "/opt/airflow";
    AIRFLOW_DAGS_DIR = "/opt/airflow/dags";
    AIRFLOW_LOGS_DIR = "/opt/airflow/logs";
    AIRFLOW_SCHEDULER_LOGS_DIR = "/opt/airflow/logs/scheduler";
    AIRFLOW_PLUGINS_DIR = "/opt/airflow/plugins";
    AIRFLOW_TMP_DIR = "/opt/airflow/tmp";
    AIRFLOW_CONF_FILE = "/opt/airflow/airflow.cfg";
    AIRFLOW_WEBSERVER_CONF_FILE = "/opt/airflow/webserver_config.py";

    # User configuration
    AIRFLOW_USERNAME = "admin";
    AIRFLOW_PASSWORD = "admin";
    AIRFLOW_FIRSTNAME = "Firstname";
    AIRFLOW_LASTNAME = "Lastname";
    AIRFLOW_EMAIL = "user@example.com";

    # Component configuration
    AIRFLOW_COMPONENT_TYPE = "api-server";
    AIRFLOW_EXECUTOR = "LocalExecutor";
    AIRFLOW_SKIP_DB_SETUP = "no";
    AIRFLOW_LOAD_EXAMPLES = "yes";
    AIRFLOW_STANDALONE_DAG_PROCESSOR = "no";
    AIRFLOW_DB_MIGRATE_TIMEOUT = "120";
    AIRFLOW_FORCE_OVERWRITE_CONF_FILE = "no";

    # API Server / Webserver configuration
    AIRFLOW_APISERVER_HOST = "127.0.0.1";
    AIRFLOW_APISERVER_PORT_NUMBER = "8080";
    AIRFLOW_WEBSERVER_SECRET_KEY = "airflow-web-server-key";
    AIRFLOW_APISERVER_SECRET_KEY = "airflow-api-server-key";
    AIRFLOW_JWT_SECRET_KEY = "";  # Auto-generated if empty
    AIRFLOW_ENABLE_HTTPS = "no";
    AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER = "80";

    # Triggerer configuration
    AIRFLOW_TRIGGERER_DEFAULT_CAPACITY = "1000";

    # Database configuration
    AIRFLOW_DATABASE_HOST = "postgresql";
    AIRFLOW_DATABASE_PORT_NUMBER = "5432";
    AIRFLOW_DATABASE_NAME = "airflow";
    AIRFLOW_DATABASE_USERNAME = "airflow";
    AIRFLOW_DATABASE_USE_SSL = "no";
    AIRFLOW_DATABASE_PASSWORD = "";

    # Redis (Celery) configuration
    REDIS_HOST = "redis";
    REDIS_PORT_NUMBER = "6379";
    REDIS_DATABASE = "1";
    REDIS_PASSWORD = "";
    AIRFLOW_REDIS_USE_SSL = "no";

    # LDAP configuration
    AIRFLOW_LDAP_ENABLE = "no";
    AIRFLOW_LDAP_USER_REGISTRATION = "True";
    AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN = "True";
    AIRFLOW_LDAP_USE_TLS = "False";
    AIRFLOW_LDAP_ALLOW_SELF_SIGNED = "True";

    # Python cache
    PYTHONPYCACHEPREFIX = "/opt/airflow/tmp/pycache";

    # Debug mode
    BITNAMI_DEBUG = "false";
  };

  # Variables that support Docker secrets (_FILE suffix)
  envVarsWithSecrets = [
    "AIRFLOW_USERNAME"
    "AIRFLOW_PASSWORD"
    "AIRFLOW_FIRSTNAME"
    "AIRFLOW_LASTNAME"
    "AIRFLOW_EMAIL"
    "AIRFLOW_COMPONENT_TYPE"
    "AIRFLOW_EXECUTOR"
    "AIRFLOW_RAW_FERNET_KEY"
    "AIRFLOW_FORCE_OVERWRITE_CONF_FILE"
    "AIRFLOW_FERNET_KEY"
    "AIRFLOW_WEBSERVER_SECRET_KEY"
    "AIRFLOW_APISERVER_SECRET_KEY"
    "AIRFLOW_JWT_SECRET_KEY"
    "AIRFLOW_APISERVER_BASE_URL"
    "AIRFLOW_APISERVER_HOST"
    "AIRFLOW_APISERVER_PORT_NUMBER"
    "AIRFLOW_LOAD_EXAMPLES"
    "AIRFLOW_HOSTNAME_CALLABLE"
    "AIRFLOW_POOL_NAME"
    "AIRFLOW_POOL_SIZE"
    "AIRFLOW_POOL_DESC"
    "AIRFLOW_STANDALONE_DAG_PROCESSOR"
    "AIRFLOW_TRIGGERER_DEFAULT_CAPACITY"
    "AIRFLOW_WORKER_QUEUE"
    "AIRFLOW_SKIP_DB_SETUP"
    "AIRFLOW_DB_MIGRATE_TIMEOUT"
    "AIRFLOW_ENABLE_HTTPS"
    "AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER"
    "AIRFLOW_DATABASE_HOST"
    "AIRFLOW_DATABASE_PORT_NUMBER"
    "AIRFLOW_DATABASE_NAME"
    "AIRFLOW_DATABASE_USERNAME"
    "AIRFLOW_DATABASE_PASSWORD"
    "AIRFLOW_DATABASE_USE_SSL"
    "AIRFLOW_REDIS_USE_SSL"
    "REDIS_HOST"
    "REDIS_PORT_NUMBER"
    "REDIS_USER"
    "REDIS_PASSWORD"
    "REDIS_DATABASE"
    "AIRFLOW_LDAP_ENABLE"
    "AIRFLOW_LDAP_URI"
    "AIRFLOW_LDAP_SEARCH"
    "AIRFLOW_LDAP_UID_FIELD"
    "AIRFLOW_LDAP_BIND_USER"
    "AIRFLOW_LDAP_BIND_PASSWORD"
    "AIRFLOW_LDAP_USER_REGISTRATION"
    "AIRFLOW_LDAP_USER_REGISTRATION_ROLE"
    "AIRFLOW_LDAP_ROLES_MAPPING"
    "AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN"
    "AIRFLOW_LDAP_USE_TLS"
    "AIRFLOW_LDAP_ALLOW_SELF_SIGNED"
    "AIRFLOW_LDAP_TLS_CA_CERTIFICATE"
  ];

  # Runtime directories with declarative schema
  # These are created at container startup with proper permissions
  # Uses numeric UID/GID (1001) for portability - no /etc/passwd dependency
  runtimeDirs = {
    home = {
      path = "/opt/airflow";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow home directory";
    };
    dags = {
      path = "/opt/airflow/dags";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "DAG definition files";
    };
    plugins = {
      path = "/opt/airflow/plugins";
      type = "plugins";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow plugins";
    };
    logs = {
      path = "/opt/airflow/logs";
      type = "logs";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow task and scheduler logs";
    };
    schedulerLogs = {
      path = "/opt/airflow/logs/scheduler";
      type = "logs";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Scheduler-specific logs";
    };
    tmp = {
      path = "/opt/airflow/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files, cleared on restart";
    };
    pycache = {
      path = "/opt/airflow/tmp/pycache";
      type = "cache";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Python bytecode cache";
    };
    bitnamiPython = {
      path = "/bitnami/python";
      type = "custom";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Bitnami Python compatibility directory";
    };
    state = {
      path = "/firestream/airflow/.state";
      type = "state";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Application state tracking (config hashes, generation counter)";
    };
  };

  # Build-time: Static config templates
  prepopulateFiles = {
    "/opt/airflow/airflow.cfg.template" = airflowConfigTemplate;
    "/opt/airflow/webserver_config.py.template" = webserverConfigTemplate;
  };

  # Validation function (from validate.sh)
  # Prepend airflow helpers so secret processing functions are available
  validateFn = airflowHelpers + validateScript;

  # Activation: Replace {{PLACEHOLDERS}} with runtime values
  # This runs after validation but before init/config
  activateFn = ''
    info "Activating Airflow configuration..."

    # Build computed values (use :- for optional/secret variables)
    local db_user db_pass db_ssl_opt scheme base_url
    db_user=$(airflow_encode_url "''${AIRFLOW_DATABASE_USERNAME:-airflow}")
    db_pass=$(airflow_encode_url "''${AIRFLOW_DATABASE_PASSWORD:-}")
    db_ssl_opt=""
    is_boolean_yes "''${AIRFLOW_DATABASE_USE_SSL:-no}" && db_ssl_opt="?sslmode=require"

    # Compute database connection string
    local sql_alchemy_conn="postgresql+psycopg2://''${db_user}:''${db_pass}@''${AIRFLOW_DATABASE_HOST:-localhost}:''${AIRFLOW_DATABASE_PORT_NUMBER:-5432}/''${AIRFLOW_DATABASE_NAME:-airflow}''${db_ssl_opt}"

    # Compute base URL
    scheme="http"
    is_boolean_yes "''${AIRFLOW_ENABLE_HTTPS:-no}" && scheme="https"
    base_url="''${scheme}://''${AIRFLOW_APISERVER_HOST:-localhost}"
    local port="''${AIRFLOW_APISERVER_PORT_NUMBER:-8080}"
    [[ "$port" != "80" && "$port" != "443" ]] && \
      base_url="''${base_url}:''${port}"

    # Compute execution API URL (Airflow 3 scheduler needs this)
    local execution_api_url="http://''${AIRFLOW_APISERVER_HOST:-localhost}:''${port}/execution/"

    # Compute Celery broker URL if needed
    local celery_broker_url="" celery_result_backend=""
    local executor="''${AIRFLOW_EXECUTOR:-LocalExecutor}"
    if [[ "$executor" == "CeleryExecutor" || "$executor" == "CeleryKubernetesExecutor" ]]; then
      local redis_user redis_pass redis_proto
      redis_user=$(airflow_encode_url "''${REDIS_USER:-}")
      redis_pass=$(airflow_encode_url "''${REDIS_PASSWORD:-}")
      redis_proto="redis"
      is_boolean_yes "''${AIRFLOW_REDIS_USE_SSL:-no}" && redis_proto="rediss"

      if [[ -n "''${REDIS_USER:-}" ]]; then
        celery_broker_url="''${redis_proto}://''${redis_user}:''${redis_pass}@''${REDIS_HOST:-redis}:''${REDIS_PORT_NUMBER:-6379}/''${REDIS_DATABASE:-0}"
      else
        celery_broker_url="''${redis_proto}://:''${redis_pass}@''${REDIS_HOST:-redis}:''${REDIS_PORT_NUMBER:-6379}/''${REDIS_DATABASE:-0}"
      fi
      celery_result_backend="db+postgresql://''${db_user}:''${db_pass}@''${AIRFLOW_DATABASE_HOST:-localhost}:''${AIRFLOW_DATABASE_PORT_NUMBER:-5432}/''${AIRFLOW_DATABASE_NAME:-airflow}''${db_ssl_opt}"
    fi

    # Compute load_examples and standalone_dag_processor
    local load_examples_val="False" standalone_dag_val="False" logging_level="INFO"
    is_boolean_yes "''${AIRFLOW_LOAD_EXAMPLES:-no}" && load_examples_val="True"
    is_boolean_yes "''${AIRFLOW_STANDALONE_DAG_PROCESSOR:-no}" && standalone_dag_val="True"
    is_boolean_yes "''${BITNAMI_DEBUG:-false}" && logging_level="DEBUG"

    # Process config template - replace placeholders with runtime values
    if [[ -f "''${AIRFLOW_HOME:-/opt/airflow}/airflow.cfg.template" ]]; then
      ${pkgs.gnused}/bin/sed \
        -e "s|{{AIRFLOW_EXECUTOR}}|''${executor}|g" \
        -e "s|{{AIRFLOW_LOAD_EXAMPLES}}|''${load_examples_val}|g" \
        -e "s|{{AIRFLOW_FERNET_KEY}}|''${AIRFLOW_FERNET_KEY:-}|g" \
        -e "s|{{AIRFLOW_DATABASE_SQL_ALCHEMY_CONN}}|''${sql_alchemy_conn}|g" \
        -e "s|{{AIRFLOW_APISERVER_PORT_NUMBER}}|''${port}|g" \
        -e "s|{{AIRFLOW_APISERVER_HOST}}|''${AIRFLOW_APISERVER_HOST:-localhost}|g" \
        -e "s|{{AIRFLOW_BASE_URL}}|''${base_url}|g" \
        -e "s|{{AIRFLOW_EXECUTION_API_SERVER_URL}}|''${execution_api_url}|g" \
        -e "s|{{AIRFLOW_WEBSERVER_SECRET_KEY}}|''${AIRFLOW_WEBSERVER_SECRET_KEY:-}|g" \
        -e "s|{{AIRFLOW_APISERVER_SECRET_KEY}}|''${AIRFLOW_APISERVER_SECRET_KEY:-}|g" \
        -e "s|{{AIRFLOW_JWT_SECRET_KEY}}|''${AIRFLOW_JWT_SECRET_KEY:-}|g" \
        -e "s|{{AIRFLOW_STANDALONE_DAG_PROCESSOR}}|''${standalone_dag_val}|g" \
        -e "s|{{AIRFLOW_CELERY_BROKER_URL}}|''${celery_broker_url}|g" \
        -e "s|{{AIRFLOW_CELERY_RESULT_BACKEND}}|''${celery_result_backend}|g" \
        -e "s|{{AIRFLOW_TRIGGERER_DEFAULT_CAPACITY}}|''${AIRFLOW_TRIGGERER_DEFAULT_CAPACITY:-1000}|g" \
        -e "s|{{AIRFLOW_LOGGING_LEVEL}}|''${logging_level}|g" \
        "''${AIRFLOW_HOME:-/opt/airflow}/airflow.cfg.template" > "''${AIRFLOW_HOME:-/opt/airflow}/airflow.cfg"

      info "Generated airflow.cfg from template"
    fi

    # Process webserver config template
    local home="''${AIRFLOW_HOME:-/opt/airflow}"
    local webserver_conf="''${AIRFLOW_WEBSERVER_CONF_FILE:-$home/webserver_config.py}"
    if [[ -f "$home/webserver_config.py.template" ]] && [[ ! -f "$webserver_conf" ]]; then
      cp "$home/webserver_config.py.template" "$webserver_conf"
      info "Generated webserver_config.py from template"
    fi

    # Save config hash for change detection (using state module)
    save_config_hash "airflow" "$home/airflow.cfg"

    info "Airflow configuration activated"
  '';

  # Initialization (database, users, pools) - from init.sh
  # Prepend airflow helpers so functions are available
  initFn = airflowHelpers + initScript;

  # Runtime config adjustments (crudini-based) - from config.sh
  # Prepend airflow helpers so functions are available
  configFn = airflowHelpers + configScript;

  # Startup command based on component type
  runCmd = ''
    ${airflowHelpers}
    # Determine Airflow major version
    major_version=$(airflow_major_version)

    case "''${AIRFLOW_COMPONENT_TYPE:-api-server}" in
      api-server)
        if [[ $major_version -eq 2 ]]; then
          info "Starting Airflow webserver (v2)..."
          cmd=(airflow webserver)
        else
          info "Starting Airflow API server (v3+)..."
          cmd=(airflow api-server)
        fi
        ;;
      webserver)
        info "Starting Airflow webserver..."
        cmd=(airflow webserver)
        ;;
      scheduler)
        info "Starting Airflow scheduler..."
        cmd=(airflow scheduler)
        ;;
      triggerer)
        info "Starting Airflow triggerer..."
        cmd=(airflow triggerer)
        ;;
      dag-processor)
        info "Starting Airflow DAG processor..."
        cmd=(airflow dag-processor)
        ;;
      worker)
        info "Starting Airflow Celery worker..."
        cmd=(airflow celery worker)
        [[ -n "''${AIRFLOW_WORKER_QUEUE:-}" ]] && cmd+=(-q "$AIRFLOW_WORKER_QUEUE")
        ;;
      *)
        error "Unknown component type: ''${AIRFLOW_COMPONENT_TYPE}"
        exit 1
        ;;
    esac

    # Add PID file
    cmd+=(--pid "''${AIRFLOW_TMP_DIR}/airflow-''${AIRFLOW_COMPONENT_TYPE}.pid")

    info "Executing: ''${cmd[*]}"
    exec "''${cmd[@]}"
  '';

  inherit systemDeps runtimeBinDeps;

  exposedPorts = [ 8080 8125 8793 8794 ];
  volumes = [ "/opt/airflow/dags" "/opt/airflow/logs" "/opt/airflow/plugins" ];

  # Python-specific options
  compileByteCode = false;  # Airflow has many packages, skip compilation for faster builds
  requirementsPath = "/bitnami/python/requirements.txt";
  enablePip = true;

  # Development shell extras
  devShellPackages = with pkgs; [ uv docker docker-compose ];
  devShellHook = ''
    echo "Airflow Version: ${airflowVersion}"
    echo ""
    echo "Build commands:"
    echo "  nix build .#dockerImage    - Build the Docker image"
    echo "  docker load < result       - Load image into Docker"
  '';
}
