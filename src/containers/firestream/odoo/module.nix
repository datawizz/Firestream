# Odoo Container Module - Using Firestream Factories
# Copyright Firestream. MIT License.
#
# This module defines the Odoo container using mkPythonContainerModule.
# Much simpler than manual implementation - the factory handles:
# - Entrypoint generation
# - Docker image building
# - Environment loading and secrets
# - Development shell
#
# Usage:
#   odooModule = import ./module.nix {
#     inherit pkgs lib firestream pythonEnv odooVersion python odooSource;
#   };

{ pkgs
, lib
, firestream
, pythonEnv          # The virtual environment from uv2nix
, odooVersion        # e.g., "18.0"
, python ? pkgs.python312
, odooSource         # The Odoo source derivation

# Externalized core-surface config. Defaults below are EXACTLY today's literals
# so the legacy flake.nix path (which does not pass these) and evalContainer
# (which passes the same values from options.nix) yield identical factory args.

# Paths configuration
, paths ? {
    base = "/opt/odoo";
    conf = "/opt/odoo/conf";
    data = "/opt/odoo/data";
    logs = "/opt/odoo/log";
  }

# Environment variables with defaults
, envVars ? {
    # Paths
    ODOO_BASE_DIR = "/opt/odoo";
    ODOO_BIN_DIR = "/opt/odoo/bin";
    ODOO_CONF_DIR = "/opt/odoo/conf";
    ODOO_CONF_FILE = "/opt/odoo/conf/odoo.conf";
    ODOO_DATA_DIR = "/opt/odoo/data";
    ODOO_ADDONS_DIR = "/opt/odoo/addons";
    ODOO_TMP_DIR = "/opt/odoo/tmp";
    ODOO_PID_FILE = "/opt/odoo/tmp/odoo.pid";
    ODOO_LOGS_DIR = "/opt/odoo/log";
    ODOO_LOG_FILE = "/opt/odoo/log/odoo-server.log";

    # Volume paths
    ODOO_VOLUME_DIR = "/opt/odoo";

    # User/group
    ODOO_DAEMON_USER = "odoo";
    ODOO_DAEMON_GROUP = "odoo";

    # Port configuration
    ODOO_PORT_NUMBER = "8069";
    ODOO_LONGPOLLING_PORT_NUMBER = "8072";

    # Bootstrap configuration
    ODOO_SKIP_BOOTSTRAP = "no";
    ODOO_SKIP_MODULES_UPDATE = "no";
    ODOO_LOAD_DEMO_DATA = "no";
    ODOO_LIST_DB = "no";

    # Odoo credentials
    ODOO_EMAIL = "admin";
    ODOO_PASSWORD = "admin";

    # SMTP configuration
    ODOO_SMTP_HOST = "";
    ODOO_SMTP_PORT_NUMBER = "";
    ODOO_SMTP_USER = "";
    ODOO_SMTP_PASSWORD = "";
    ODOO_SMTP_PROTOCOL = "";

    # Database configuration
    ODOO_DATABASE_HOST = "postgresql";
    ODOO_DATABASE_PORT_NUMBER = "5432";
    ODOO_DATABASE_NAME = "firestream_odoo";
    ODOO_DATABASE_USER = "firestream";
    ODOO_DATABASE_PASSWORD = "";
    ODOO_DATABASE_FILTER = "";

    # Timeouts
    ODOO_DB_WAIT_TIMEOUT = "120";

    # Empty password flag
    ALLOW_EMPTY_PASSWORD = "no";

    # Debug mode
    BITNAMI_DEBUG = "false";

    # Odoo version (for scripts)
    ODOO_VERSION = odooVersion;
  }

# Variables that support Docker secrets (_FILE suffix)
, envVarsWithSecrets ? [
    "ODOO_PASSWORD"
    "ODOO_DATABASE_PASSWORD"
    "ODOO_SMTP_PASSWORD"
    "ODOO_SMTP_HOST"
    "ODOO_SMTP_PORT_NUMBER"
    "ODOO_SMTP_USER"
    "ODOO_SMTP_PROTOCOL"
    "ODOO_DATABASE_HOST"
    "ODOO_DATABASE_PORT_NUMBER"
    "ODOO_DATABASE_NAME"
    "ODOO_DATABASE_USER"
    "ODOO_DATABASE_FILTER"
    "ODOO_EMAIL"
    "ODOO_SKIP_BOOTSTRAP"
    "ODOO_SKIP_MODULES_UPDATE"
    "ODOO_LOAD_DEMO_DATA"
    "ODOO_LIST_DB"
  ]

, exposedPorts ? [ 8069 8072 ]

# Image naming passthrough (parity defaults).
, imageName ? "firestream-odoo"
, imageTag ? odooVersion
}:

let
  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;

  # Odoo-specific helper functions (needed by scripts)
  odooHelpers = ''
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
    # Ensure directory exists
    # Arguments:
    #   $1 - directory path
    #########################
    ensure_dir_exists() {
      local dir="$1"
      [[ -d "$dir" ]] || mkdir -p "$dir"
    }

    ########################
    # Wait for PostgreSQL to be available
    # Uses environment variables for connection details
    # Returns:
    #   0 if connection succeeded, 1 otherwise
    #########################
    odoo_wait_for_postgresql() {
      local host="''${ODOO_DATABASE_HOST:-postgresql}"
      local port="''${ODOO_DATABASE_PORT_NUMBER:-5432}"
      local timeout="''${ODOO_DB_WAIT_TIMEOUT:-120}"

      info "Waiting for PostgreSQL at $host:$port (timeout: $timeout seconds)..."

      if wait-for-port --host "$host" --timeout "$timeout" "$port"; then
        info "PostgreSQL is available at $host:$port"
        return 0
      else
        error "Timeout waiting for PostgreSQL at $host:$port"
        return 1
      fi
    }

    ########################
    # Execute SQL against the Odoo database
    # Arguments:
    #   $1 - SQL statement
    # Returns:
    #   Query result
    #########################
    odoo_db_execute() {
      local sql="''${1:?SQL statement required}"
      local host="''${ODOO_DATABASE_HOST:-postgresql}"
      local port="''${ODOO_DATABASE_PORT_NUMBER:-5432}"
      local db="''${ODOO_DATABASE_NAME:-firestream_odoo}"
      local user="''${ODOO_DATABASE_USER:-firestream}"

      PGPASSWORD="''${ODOO_DATABASE_PASSWORD:-}" ${pkgs.postgresql}/bin/psql \
        -h "$host" \
        -p "$port" \
        -U "$user" \
        -d "$db" \
        -t -c "$sql"
    }

    ########################
    # Execute Odoo CLI command
    # Runs odoo-bin with the current configuration
    # Arguments:
    #   $@ - Arguments to pass to odoo-bin
    #########################
    odoo_execute() {
      local config="''${ODOO_CONF_FILE:-/opt/odoo/conf/odoo.conf}"

      debug "Executing: python /opt/odoo/odoo-bin --config=$config --logfile= --pidfile= --stop-after-init $*"
      python /opt/odoo/odoo-bin \
        --config="$config" \
        --logfile= \
        --pidfile= \
        --stop-after-init \
        "$@"
    }

    ########################
    # Set a configuration value in odoo.conf
    # Arguments:
    #   $1 - key
    #   $2 - value
    #########################
    odoo_conf_set() {
      local key="''${1:?key required}"
      local value="''${2:?value required}"
      local conf_file="''${ODOO_CONF_FILE:-/opt/odoo/conf/odoo.conf}"

      debug "Setting $key = $value in $conf_file"

      # Escape special characters for sed
      local escaped_key escaped_value
      escaped_key=$(printf '%s\n' "$key" | ${pkgs.gnused}/bin/sed 's/[[\.*^$()+?{|]/\\&/g')
      escaped_value=$(printf '%s\n' "$value" | ${pkgs.gnused}/bin/sed 's/[[\.*^$()+?{|]/\\&/g')

      if grep -q "^[[:space:]]*;*[[:space:]]*$escaped_key[[:space:]]*=" "$conf_file" 2>/dev/null; then
        # Key exists, replace it
        ${pkgs.gnused}/bin/sed -i "s|^[[:space:]]*;*[[:space:]]*$escaped_key[[:space:]]*=.*|$key = $value|" "$conf_file"
      else
        # Key doesn't exist, append it
        echo "$key = $value" >> "$conf_file"
      fi
    }

    ########################
    # Get a configuration value from odoo.conf
    # Arguments:
    #   $1 - key
    # Returns:
    #   Configuration value
    #########################
    odoo_conf_get() {
      local key="''${1:?key required}"
      local conf_file="''${ODOO_CONF_FILE:-/opt/odoo/conf/odoo.conf}"

      grep "^[[:space:]]*$key[[:space:]]*=" "$conf_file" 2>/dev/null | \
        ${pkgs.gnused}/bin/sed "s|^[[:space:]]*$key[[:space:]]*=[[:space:]]*||" | \
        tr -d "\"' "
    }

    ########################
    # Get Odoo major version
    # Returns:
    #   Major version number (e.g., 18)
    #########################
    odoo_major_version() {
      python /opt/odoo/odoo-bin --version 2>/dev/null | \
        grep -E -o "[0-9]+\.[0-9]+\.[0-9]+" | \
        cut -d'.' -f1
    }
  '';

  # System dependencies (libs needed in the container)
  systemDeps = with pkgs; [
    # SSL/TLS and crypto
    cacert openssl

    # LDAP
    openldap cyrus_sasl

    # Database clients
    postgresql

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

    # Image processing (for Pillow and PDF generation)
    imagemagick
    libjpeg
    libpng
    freetype
    fontconfig

    # XML processing
    libxml2
    libxslt

    # PDF generation
    wkhtmltopdf

    # Node.js tools for Odoo assets
    nodejs_22
    nodePackages.rtlcss
    nodePackages.less

    # Fonts (required for proper PDF rendering)
    dejavu_fonts
    liberation_ttf
    noto-fonts
    freefont_ttf

    # C++ runtime
    stdenv.cc.cc.lib
  ];

  # Runtime binary deps (need to be in PATH)
  runtimeBinDeps = with pkgs; [
    coreutils bash gnused gnugrep gawk findutils which
    postgresql git curl netcat-gnu
    wkhtmltopdf
    nodejs_22
    nodePackages.rtlcss
    nodePackages.less
    fontconfig
    firestream.waitForPortPkg  # Required by init scripts for database readiness checks
  ];

  # Odoo config template with {{PLACEHOLDER}} syntax
  odooConfigTemplate = ''
    [options]
    ; Addons paths
    addons_path = /opt/odoo/addons,/opt/odoo/odoo/addons,{{ODOO_ADDONS_DIR}}

    ; Admin password for database management
    admin_passwd = {{ODOO_PASSWORD}}

    ; Data directory
    data_dir = {{ODOO_DATA_DIR}}

    ; Log file
    logfile = {{ODOO_LOG_FILE}}

    ; Database connection
    db_host = {{ODOO_DATABASE_HOST}}
    db_name = {{ODOO_DATABASE_NAME}}
    db_password = {{ODOO_DATABASE_PASSWORD}}
    db_port = {{ODOO_DATABASE_PORT_NUMBER}}
    db_user = {{ODOO_DATABASE_USER}}

    ; HTTP configuration
    http_port = {{ODOO_PORT_NUMBER}}
    gevent_port = {{ODOO_LONGPOLLING_PORT_NUMBER}}

    ; Performance
    limit_time_cpu = 90
    limit_time_real = 150
    max_cron_threads = 1

    ; Security
    list_db = {{ODOO_LIST_DB}}
    proxy_mode = True

    ; Logging
    log_level = {{ODOO_LOG_LEVEL}}
  '';

in firestream.mkPythonContainerModule {
  name = "odoo";
  version = odooVersion;
  inherit pythonEnv python;

  # Paths, environment variables, and secret-aware variables are externalized
  # as function arguments (defaults equal to the historical literals). The
  # legacy flake.nix path uses the defaults; evalContainer passes the same
  # values from options.nix, yielding identical factory args.
  inherit paths envVars envVarsWithSecrets;

  # Image naming passthrough.
  inherit imageName imageTag;

  # Runtime directories with declarative schema
  runtimeDirs = {
    home = {
      path = "/opt/odoo";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Odoo home directory";
    };
    conf = {
      path = "/opt/odoo/conf";
      type = "conf";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Odoo configuration directory";
    };
    data = {
      path = "/opt/odoo/data";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Odoo data/filestore directory";
    };
    addons = {
      path = "/opt/odoo/addons";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Custom Odoo addons/modules";
    };
    logs = {
      path = "/opt/odoo/log";
      type = "logs";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Odoo log files";
    };
    tmp = {
      path = "/opt/odoo/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files";
    };
    volume = {
      path = "/opt/odoo";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Odoo volume mount point";
    };
    bitnamiPython = {
      path = "/bitnami/python";
      type = "custom";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Custom Python requirements directory";
    };
    state = {
      path = "/firestream/odoo/.state";
      type = "state";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Application state tracking";
    };
  };

  # Build-time: Static config templates
  prepopulateFiles = {
    "/opt/odoo/conf/odoo.conf.template" = odooConfigTemplate;
  };

  # Validation function
  validateFn = odooHelpers + ''
    error_code=0
    ${validateScript}
    [[ "$error_code" -eq 0 ]] || exit "$error_code"
  '';

  # Activation: Load secrets and process config templates
  activateFn = odooHelpers + ''
    info "Activating Odoo configuration..."

    # Load secrets from _FILE variables
    ${secretsScript}

    # Compute list_db and log_level values
    local list_db_val log_level_val
    list_db_val="$(is_boolean_yes "$ODOO_LIST_DB" && echo 'True' || echo 'False')"
    log_level_val="$(is_boolean_yes "$BITNAMI_DEBUG" && echo 'debug' || echo 'info')"

    # Process config template if exists
    local conf_dir="''${ODOO_CONF_DIR:-/opt/odoo/conf}"
    local conf_file="''${ODOO_CONF_FILE:-$conf_dir/odoo.conf}"

    if [[ -f "$conf_dir/odoo.conf.template" ]] && [[ ! -f "$conf_file" ]]; then
      info "Generating odoo.conf from template..."
      ${pkgs.gnused}/bin/sed \
        -e "s|{{ODOO_ADDONS_DIR}}|''${ODOO_ADDONS_DIR:-/opt/odoo/addons}|g" \
        -e "s|{{ODOO_PASSWORD}}|''${ODOO_PASSWORD:-admin}|g" \
        -e "s|{{ODOO_DATA_DIR}}|''${ODOO_DATA_DIR:-/opt/odoo/data}|g" \
        -e "s|{{ODOO_LOG_FILE}}|''${ODOO_LOG_FILE:-/opt/odoo/log/odoo-server.log}|g" \
        -e "s|{{ODOO_DATABASE_HOST}}|''${ODOO_DATABASE_HOST:-postgresql}|g" \
        -e "s|{{ODOO_DATABASE_NAME}}|''${ODOO_DATABASE_NAME:-firestream_odoo}|g" \
        -e "s|{{ODOO_DATABASE_PASSWORD}}|''${ODOO_DATABASE_PASSWORD:-}|g" \
        -e "s|{{ODOO_DATABASE_PORT_NUMBER}}|''${ODOO_DATABASE_PORT_NUMBER:-5432}|g" \
        -e "s|{{ODOO_DATABASE_USER}}|''${ODOO_DATABASE_USER:-firestream}|g" \
        -e "s|{{ODOO_PORT_NUMBER}}|''${ODOO_PORT_NUMBER:-8069}|g" \
        -e "s|{{ODOO_LONGPOLLING_PORT_NUMBER}}|''${ODOO_LONGPOLLING_PORT_NUMBER:-8072}|g" \
        -e "s|{{ODOO_LIST_DB}}|''${list_db_val}|g" \
        -e "s|{{ODOO_LOG_LEVEL}}|''${log_level_val}|g" \
        "$conf_dir/odoo.conf.template" > "$conf_file"

      info "Generated odoo.conf"
    fi

    # Save config hash for change detection
    save_config_hash "odoo" "$conf_file"

    info "Odoo configuration activated"
  '';

  # Configuration generation
  configFn = odooHelpers + configScript;

  # Initialization
  initFn = odooHelpers + initScript;

  # Startup command
  runCmd = ''
    ${odooHelpers}

    info "Starting Odoo ${odooVersion}..."

    conf_file="''${ODOO_CONF_FILE:-/opt/odoo/conf/odoo.conf}"
    pid_file="''${ODOO_PID_FILE:-/opt/odoo/tmp/odoo.pid}"

    # Run Odoo
    exec python /opt/odoo/odoo-bin \
      --config="$conf_file" \
      --pidfile="$pid_file"
  '';

  inherit systemDeps runtimeBinDeps;

  inherit exposedPorts;
  volumes = [ "/opt/odoo/data" "/opt/odoo/addons" "/bitnami/python" "/docker-entrypoint-init.d" ];

  user = {
    name = "odoo";
    group = "odoo";
    uid = 1001;
    gid = 1001;
  };

  # Python-specific options
  compileByteCode = false;
  requirementsPath = "/bitnami/python/requirements.txt";
  enablePip = true;

  # Extra packages for the container
  extraDeps = [ odooSource ];

  # Development shell extras
  devShellPackages = with pkgs; [ uv docker docker-compose ];
  devShellHook = ''
    echo "Odoo Version: ${odooVersion}"
    echo ""
    echo "Build commands:"
    echo "  nix build .#dockerImage    - Build the Docker image"
    echo "  docker load < result       - Load image into Docker"
  '';
}
