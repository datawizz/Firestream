# Airflow Container Module - Using Firestream Factories
# Copyright Firestream. MIT License.
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

# Externalized core-surface config. Defaults below are EXACTLY today's literals
# so the legacy flake.nix path (which does not pass these) and evalContainer
# (which passes the same values from options.nix) yield identical factory args.
# NOTE: these defaults are intentionally duplicated in options.nix for Phase 2;
# a later phase removes the duplication. Keep them identical.

# Paths configuration (Airflow uses /opt/firestream/airflow structure)
, paths ? {
    base = "/opt/firestream/airflow";
    conf = "/opt/firestream/airflow";
    data = "/opt/firestream/airflow/dags";
    logs = "/opt/firestream/airflow/logs";
  }

# Environment variables with defaults (from env-defaults.sh)
, envVars ? {
    # Paths
    AIRFLOW_HOME = "/opt/firestream/airflow";
    AIRFLOW_DAGS_DIR = "/opt/firestream/airflow/dags";
    AIRFLOW_LOGS_DIR = "/opt/firestream/airflow/logs";
    AIRFLOW_SCHEDULER_LOGS_DIR = "/opt/firestream/airflow/logs/scheduler";
    AIRFLOW_PLUGINS_DIR = "/opt/firestream/airflow/plugins";
    AIRFLOW_TMP_DIR = "/opt/firestream/airflow/tmp";
    AIRFLOW_CONF_FILE = "/opt/firestream/airflow/airflow.cfg";
    AIRFLOW_WEBSERVER_CONF_FILE = "/opt/firestream/airflow/webserver_config.py";

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
    PYTHONPYCACHEPREFIX = "/opt/firestream/airflow/tmp/pycache";

    # Debug mode
    BITNAMI_DEBUG = "false";
  }

# Variables that support Docker secrets (_FILE suffix)
, envVarsWithSecrets ? [
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
  ]

, exposedPorts ? [ 8080 8125 8793 8794 ]

# In-image health/SBOM service configuration (Phase 4). Forwarded to
# mkPythonContainerModule (which forwards to mkContainerModule). Default-off
# preserves byte-identical legacy-flake behaviour.
, health ? { enable = false; port = 9180; readinessCmd = null; }

# Image naming passthrough (parity defaults).
, imageName ? "firestream-airflow"
, imageTag ? airflowVersion

# Build-time vendored DAG sources (list of specs from
# options.airflow.vendoredDags, forwarded via extraModuleArgs). Empty ⇒ no
# vendoring, stock image unchanged.
, vendoredDags ? [ ]
}:

let
  # Build the baked-DAGs derivation when any source is declared. Lays files at
  # /opt/firestream/airflow/dags (Airflow's dags_folder). See ./vendor-dags.nix.
  vendoredDagsDrv =
    import ./vendor-dags.nix { inherit pkgs lib; } {
      version = airflowVersion;
      specs = vendoredDags;
    };

  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;
  # Pure helper-function definitions extracted from init.sh + new
  # init-container helpers (airflow_conf_set, airflow_wait_for_db_*,
  # airflow_wait_for_admin_user, airflow_webserver_conf_set). Emitted at
  # top-level of libhelpersairflow.sh so chart init containers can invoke
  # them after `source /opt/firestream/scripts/libairflow.sh`.
  helpersScript = builtins.readFile ./scripts/helpers.sh;

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
    # Execute an Airflow CLI subcommand. Bitnami chart jobs/init containers
    # (setup-db, etc.) call `airflow_execute <subcommand>` after sourcing
    # libairflow.sh. The firestream container already runs as the airflow user
    # with PATH/PYTHONPATH + airflow.cfg in place, so this is a thin wrapper.
    # Arguments:
    #   $@ - airflow CLI args (e.g. "db migrate", "sync-perm --include-dags")
    ########################
    airflow_execute() {
        airflow "$@"
    }

    ########################
    # Create the Airflow admin (FAB) user from AIRFLOW_USERNAME/PASSWORD/etc.
    # Idempotent: a pre-existing user makes `airflow users create` warn, which
    # we tolerate. Mirrors the inline logic in scripts/init.sh so the chart's
    # setup-db job (which calls airflow_create_admin_user) behaves identically.
    # Arguments:
    #   None (reads AIRFLOW_USERNAME/PASSWORD/EMAIL/FIRSTNAME/LASTNAME)
    ########################
    airflow_create_admin_user() {
        if [[ -z "''${AIRFLOW_USERNAME:-}" || -z "''${AIRFLOW_PASSWORD:-}" ]]; then
            debug "AIRFLOW_USERNAME/PASSWORD unset - skipping admin user creation"
            return 0
        fi
        if airflow users list --output plain 2>&1 | grep -v DEBUG | grep -q "''${AIRFLOW_USERNAME}"; then
            info "Airflow admin user ''${AIRFLOW_USERNAME} already exists"
            return 0
        fi
        info "Creating Airflow admin user ''${AIRFLOW_USERNAME}..."
        airflow users create \
            -r "Admin" \
            -u "''${AIRFLOW_USERNAME}" \
            -e "''${AIRFLOW_EMAIL:-user@example.com}" \
            -p "''${AIRFLOW_PASSWORD}" \
            -f "''${AIRFLOW_FIRSTNAME:-Firstname}" \
            -l "''${AIRFLOW_LASTNAME:-Lastname}" 2>&1 || \
            warn "Failed to create admin user (it may already exist)"
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

    # ----------------------------------------------------------------------
    # Helpers relocated from scripts/init.sh + new init-container helpers
    # (airflow_conf_set, airflow_webserver_conf_set, airflow_wait_for_*)
    # required by Bitnami's chart init containers. Visible at top-level of
    # libairflow.sh so chart init containers can call them directly.
    # ----------------------------------------------------------------------
    ${helpersScript}
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
    hostname  # Celery worker readiness probe runs `celery ... -d celery@$(hostname)`
    firestream.waitForPortPkg  # Required by init.sh for database/redis readiness checks
  ];

  # Build-time: Airflow config template with {{PLACEHOLDER}} syntax
  # Runtime activation replaces these with actual environment values
  airflowConfigTemplate = ''
    # Airflow configuration template
    # Placeholders are replaced at container startup via activateFn

    [core]
    executor = {{AIRFLOW_EXECUTOR}}
    dags_folder = /opt/firestream/airflow/dags
    plugins_folder = /opt/firestream/airflow/plugins
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
    base_log_folder = /opt/firestream/airflow/logs
    logging_level = {{AIRFLOW_LOGGING_LEVEL}}

    [fab]
    config_file = /opt/firestream/airflow/webserver_config.py
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

  # Paths, environment variables, and secret-aware variables are externalized
  # as function arguments (defaults equal to the historical literals). The
  # legacy flake.nix path uses the defaults; evalContainer passes the same
  # values from options.nix, yielding identical factory args.
  inherit paths envVars envVarsWithSecrets;

  # Image naming passthrough.
  inherit imageName imageTag;

  # Build-time vendored DAGs: lay the baked /opt/firestream/airflow/dags tree
  # into the image. No-op (empty list) ⇒ no extra dep ⇒ stock image unchanged.
  extraDeps = lib.optional (vendoredDags != [ ]) vendoredDagsDrv;

  # Runtime directories with declarative schema
  # These are created at container startup with proper permissions
  # Uses numeric UID/GID (1001) for portability - no /etc/passwd dependency
  runtimeDirs = {
    home = {
      path = "/opt/firestream/airflow";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow home directory";
    };
    dags = {
      path = "/opt/firestream/airflow/dags";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "DAG definition files";
    };
    plugins = {
      path = "/opt/firestream/airflow/plugins";
      type = "plugins";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow plugins";
    };
    logs = {
      path = "/opt/firestream/airflow/logs";
      type = "logs";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Airflow task and scheduler logs";
    };
    schedulerLogs = {
      path = "/opt/firestream/airflow/logs/scheduler";
      type = "logs";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Scheduler-specific logs";
    };
    tmp = {
      path = "/opt/firestream/airflow/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files, cleared on restart";
    };
    pycache = {
      path = "/opt/firestream/airflow/tmp/pycache";
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
    "/opt/firestream/airflow/airflow.cfg.template" = airflowConfigTemplate;
    "/opt/firestream/airflow/webserver_config.py.template" = webserverConfigTemplate;
  };

  # Build-time: materialize a venv directory at /opt/firestream/airflow/venv.
  #
  # The Bitnami-derived airflow chart's `prepare-venv` init container does:
  #   cp -r --preserve=mode /opt/firestream/airflow/venv /emptydir/venv-base-dir
  # and the main containers (web/scheduler/worker/triggerer/dag-processor)
  # then remount that copy (emptyDir subPath venv-base-dir) at
  # /opt/firestream/airflow/venv, sourcing /opt/firestream/airflow/venv/bin/activate
  # and resolving the `airflow` CLI from venv/bin.
  #
  # The Nix python env is a /nix/store venv path (not at /opt/...). We expose it
  # at the chart-expected location by creating a REAL directory whose top-level
  # entries (bin, lib, pyvenv.cfg, ...) are absolute symlinks into ${pythonEnv}.
  # Why a real dir of symlinks rather than a single `venv -> ${pythonEnv}`
  # symlink:
  #   * `cp -r` copies a symlink-to-dir as a symlink, producing a broken/relative
  #     link; a real dir is recursed into and its symlink entries copied verbatim
  #     (absolute, into the in-image /nix/store).
  #   * Kubernetes refuses a subPath that is itself a symlink escaping the volume,
  #     so venv-base-dir must be a real directory; symlinks *inside* it are fine.
  pythonPrepopulateFn = ''
    echo "Materializing airflow venv at /opt/firestream/airflow/venv -> ${pythonEnv}"
    mkdir -p "$out/opt/firestream/airflow/venv"
    for entry in ${pythonEnv}/*; do
      ln -s "$entry" "$out/opt/firestream/airflow/venv/$(basename "$entry")"
    done
  '';

  # Per-container helpers: emitted at top-level of libhelpersairflow.sh by the
  # engine, so chart init containers can `source /opt/firestream/scripts/libairflow.sh`
  # and use these helpers (airflow_conf_set, airflow_wait_for_db_connection, etc.) directly.
  perContainerHelpers = airflowHelpers;

  # Validation function (from validate.sh)
  validateFn = validateScript;

  # Activation: Replace {{PLACEHOLDERS}} with runtime values
  # This runs after validation but before init/config
  activateFn = ''
    info "Activating Airflow configuration..."

    # Chart mode: the Bitnami chart's prepare-config init container has already
    # rendered a complete airflow.cfg (correct release-scoped DB host, celery
    # broker/result backend, executor, auth manager, secrets) and mounts it
    # read-only-ish into this pod. Regenerating it here from the container's
    # baked template would clobber those values with standalone defaults (DB host
    # "postgresql", LocalExecutor, etc.) and the crudini-based configFn would
    # additionally fail trying to write a temp file in the read-only config dir.
    # So when AIRFLOW_SKIP_DB_SETUP=yes we trust the chart-provided airflow.cfg.
    if is_boolean_yes "''${AIRFLOW_SKIP_DB_SETUP:-no}"; then
      info "AIRFLOW_SKIP_DB_SETUP=yes - using chart-provided airflow.cfg; skipping in-container config generation"
      return 0
    fi

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
    if [[ -f "''${AIRFLOW_HOME:-/opt/firestream/airflow}/airflow.cfg.template" ]]; then
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
        "''${AIRFLOW_HOME:-/opt/firestream/airflow}/airflow.cfg.template" > "''${AIRFLOW_HOME:-/opt/firestream/airflow}/airflow.cfg"

      info "Generated airflow.cfg from template"
    fi

    # Process webserver config template
    local home="''${AIRFLOW_HOME:-/opt/firestream/airflow}"
    local webserver_conf="''${AIRFLOW_WEBSERVER_CONF_FILE:-$home/webserver_config.py}"
    if [[ -f "$home/webserver_config.py.template" ]] && [[ ! -f "$webserver_conf" ]]; then
      # Tolerate a read-only target: in the Bitnami chart only the web/api-server
      # pod mounts a writable webserver_config.py; scheduler/worker/triggerer/
      # dag-processor run with readOnlyRootFilesystem and do not need it. Treat a
      # write failure as a skip rather than aborting the entrypoint (EROFS).
      if cp "$home/webserver_config.py.template" "$webserver_conf" 2>/dev/null; then
        info "Generated webserver_config.py from template"
      else
        debug "Skipped webserver_config.py generation (read-only filesystem)"
      fi
    fi

    # Save config hash for change detection (using state module)
    save_config_hash "airflow" "$home/airflow.cfg"

    info "Airflow configuration activated"
  '';

  # Initialization (database, users, pools) - from init.sh
  initFn = initScript;

  # Runtime config adjustments (crudini-based) - from config.sh
  configFn = configScript;

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

  inherit exposedPorts;
  inherit health;
  volumes = [ "/opt/firestream/airflow/dags" "/opt/firestream/airflow/logs" "/opt/firestream/airflow/plugins" ];

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
