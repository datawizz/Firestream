# Redis Container Module - Using Firestream Factories
# Copyright Firestream. MIT License.
#
# This module defines the Redis container using mkContainerModule.
# The factory handles:
# - Entrypoint generation
# - Docker image building
# - Environment loading and secrets
# - Development shell
#
# Usage:
#   redisModule = import ./module.nix {
#     inherit pkgs lib firestream;
#     redisVersion = "7";  # or "8"
#   };

{ pkgs
, lib
, firestream
, redisVersion ? "7"
}:

let
  # Select Redis package based on version
  redis = if redisVersion == "8" then pkgs.redis else pkgs.redis;

  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;

  # Redis-specific helper functions (needed by scripts)
  # These supplement the core library functions
  redisHelpers = ''
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
    # Ensure directory exists
    # Arguments:
    #   $1 - directory path
    #########################
    ensure_dir_exists() {
      local dir="$1"
      [[ -d "$dir" ]] || mkdir -p "$dir"
    }

    ########################
    # Get machine IP address
    # Returns:
    #   IP address string
    #########################
    get_machine_ip() {
      hostname -I 2>/dev/null | awk '{print $1}' || echo "127.0.0.1"
    }

    ########################
    # Get Redis version
    # Returns:
    #   Redis version string
    #########################
    redis_version() {
      redis-cli --version | grep -E -o "[0-9]+\.[0-9]+\.[0-9]+"
    }

    ########################
    # Get Redis major version
    # Returns:
    #   Redis major version number
    #########################
    redis_major_version() {
      redis_version | grep -E -o "^[0-9]+"
    }

    ########################
    # Get a configuration value from redis.conf
    # Arguments:
    #   $1 - key
    #   $2 - conf file (optional)
    # Returns:
    #   Configuration value
    #########################
    redis_conf_get() {
      local key="''${1:?missing key}"
      local conf_file="''${2:-$REDIS_CONF_FILE}"

      if grep -q -E "^\s*$key " "$conf_file" 2>/dev/null; then
        grep -E "^\s*$key " "$conf_file" | awk '{print $2}'
      fi
    }

    ########################
    # Set a configuration value in redis.conf
    # Arguments:
    #   $1 - key
    #   $2 - value
    #########################
    redis_conf_set() {
      local key="''${1:?missing key}"
      local value="''${2:-}"

      # Sanitize value
      value="''${value//\\/\\\\}"
      value="''${value//&/\\&}"
      value="''${value//\?/\\?}"
      value="''${value//[$'\t\n\r']}"
      [[ -z "$value" ]] && value="\"\""

      # Special handling for 'save' directive (append instead of replace)
      if [[ "$key" == "save" ]]; then
        echo "$key $value" >> "$REDIS_CONF_FILE"
      else
        # Replace existing or append
        if grep -q -E "^#*\s*$key " "$REDIS_CONF_FILE" 2>/dev/null; then
          ${pkgs.gnused}/bin/sed -i "s|^#*\s*$key .*|$key $value|" "$REDIS_CONF_FILE"
        else
          echo "$key $value" >> "$REDIS_CONF_FILE"
        fi
      fi
    }

    ########################
    # Unset a configuration value in redis.conf
    # Arguments:
    #   $1 - key
    #########################
    redis_conf_unset() {
      local key="''${1:?missing key}"
      ${pkgs.gnused}/bin/sed -i "/^\s*$key .*/d" "$REDIS_CONF_FILE"
    }

    ########################
    # Check if Redis is running
    # Arguments:
    #   $1 - pid file (optional)
    # Returns:
    #   0 if running, 1 otherwise
    #########################
    is_redis_running() {
      local pid_file="''${1:-$REDIS_PID_FILE}"
      local pid

      [[ -f "$pid_file" ]] || return 1
      pid=$(<"$pid_file")
      [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
    }

    ########################
    # Check if Redis is not running
    # Returns:
    #   0 if not running, 1 if running
    #########################
    is_redis_not_running() {
      ! is_redis_running "$@"
    }

    ########################
    # Wait for Redis master
    # Uses REDIS_MASTER_HOST and REDIS_MASTER_PORT_NUMBER
    #########################
    redis_wait_for_master() {
      local host="''${REDIS_MASTER_HOST:-localhost}"
      local port="''${REDIS_MASTER_PORT_NUMBER:-6379}"
      local timeout=60

      info "Waiting for Redis master at $host:$port..."
      if wait-for-port --host "$host" --timeout "$timeout" "$port"; then
        info "Redis master is available"
      else
        error "Timeout waiting for Redis master at $host:$port"
        return 1
      fi
    }

    ########################
    # Disable unsafe Redis commands
    # Uses REDIS_DISABLE_COMMANDS
    #########################
    redis_disable_unsafe_commands() {
      local IFS=','
      read -ra disabledCommands <<< "$REDIS_DISABLE_COMMANDS"
      debug "Disabling commands: ''${disabledCommands[*]}"

      for cmd in "''${disabledCommands[@]}"; do
        cmd=$(echo "$cmd" | tr -d ' ')
        if grep -qE "^\s*rename-command\s+$cmd\s+\"\"\s*$" "$REDIS_CONF_FILE" 2>/dev/null; then
          debug "$cmd was already disabled"
          continue
        fi
        echo "rename-command $cmd \"\"" >> "$REDIS_CONF_FILE"
      done
    }

    ########################
    # Configure Redis permissions
    # Creates directories with proper ownership
    #########################
    redis_configure_permissions() {
      debug "Ensuring expected directories/files exist"
      for dir in "$REDIS_BASE_DIR" "$REDIS_DATA_DIR" "$REDIS_TMP_DIR" "$REDIS_LOG_DIR" "$REDIS_CONF_DIR"; do
        ensure_dir_exists "$dir"
        if [[ "$(id -u)" -eq 0 ]]; then
          chown "$REDIS_DAEMON_USER:$REDIS_DAEMON_GROUP" "$dir"
        fi
      done
    }

    ########################
    # Append include directive for overrides.conf
    #########################
    redis_append_include_conf() {
      if [[ -f "$REDIS_OVERRIDES_FILE" ]]; then
        redis_conf_unset "include"
        echo "include $REDIS_OVERRIDES_FILE" >> "$REDIS_CONF_FILE"
      fi
    }
  '';

  # System dependencies (libs needed in the container)
  systemDeps = with pkgs; [
    # SSL/TLS
    cacert openssl

    # Compression
    zlib

    # Utilities
    coreutils gnugrep gnused gawk findutils which
    procps

    # Networking
    netcat-gnu
  ];

  # Runtime binary deps (need to be in PATH)
  runtimeBinDeps = with pkgs; [
    coreutils bash gnused gnugrep gawk findutils which
    redis netcat-gnu
  ];

in firestream.mkContainerModule {
  name = "redis";
  version = redisVersion;

  # Paths configuration (Bitnami compatibility)
  paths = {
    base = "/opt/firestream/redis";
    conf = "/opt/firestream/redis/etc";
    data = "/firestream/redis/data";
    logs = "/opt/firestream/redis/logs";
  };

  # Environment variables with defaults
  envVars = {
    # Base directories
    REDIS_BASE_DIR = "/opt/firestream/redis";
    REDIS_VOLUME_DIR = "/firestream/redis";
    REDIS_DATA_DIR = "/firestream/redis/data";
    REDIS_CONF_DIR = "/opt/firestream/redis/etc";
    REDIS_DEFAULT_CONF_DIR = "/opt/firestream/redis/etc.default";
    REDIS_MOUNTED_CONF_DIR = "/opt/firestream/redis/mounted-etc";
    REDIS_OVERRIDES_FILE = "/opt/firestream/redis/mounted-etc/overrides.conf";
    REDIS_CONF_FILE = "/opt/firestream/redis/etc/redis.conf";
    REDIS_LOG_DIR = "/opt/firestream/redis/logs";
    REDIS_LOG_FILE = "/opt/firestream/redis/logs/redis.log";
    REDIS_TMP_DIR = "/opt/firestream/redis/tmp";
    REDIS_PID_FILE = "/opt/firestream/redis/tmp/redis.pid";
    REDIS_BIN_DIR = "/opt/firestream/redis/bin";

    # User and group
    REDIS_DAEMON_USER = "redis";
    REDIS_DAEMON_GROUP = "redis";

    # Connection settings
    REDIS_PORT_NUMBER = "6379";
    REDIS_ALLOW_REMOTE_CONNECTIONS = "yes";
    REDIS_EXTRA_FLAGS = "";

    # Authentication
    REDIS_PASSWORD = "";
    ALLOW_EMPTY_PASSWORD = "no";

    # Persistence
    REDIS_AOF_ENABLED = "yes";
    REDIS_RDB_POLICY = "";
    REDIS_RDB_POLICY_DISABLED = "no";

    # Replication
    REDIS_REPLICATION_MODE = "";
    REDIS_MASTER_HOST = "";
    REDIS_MASTER_PORT_NUMBER = "6379";
    REDIS_MASTER_PASSWORD = "";
    REDIS_REPLICA_IP = "";
    REDIS_REPLICA_PORT = "";

    # ACL
    REDIS_ACLFILE = "";
    REDIS_DISABLE_COMMANDS = "";

    # Multi-threading
    REDIS_IO_THREADS = "";
    REDIS_IO_THREADS_DO_READS = "";

    # TLS
    REDIS_TLS_ENABLED = "no";
    REDIS_TLS_PORT_NUMBER = "6379";
    REDIS_TLS_CERT_FILE = "";
    REDIS_TLS_KEY_FILE = "";
    REDIS_TLS_KEY_FILE_PASS = "";
    REDIS_TLS_CA_FILE = "";
    REDIS_TLS_CA_DIR = "";
    REDIS_TLS_DH_PARAMS_FILE = "";
    REDIS_TLS_AUTH_CLIENTS = "yes";

    # Sentinel
    REDIS_SENTINEL_HOST = "";
    REDIS_SENTINEL_PORT_NUMBER = "26379";
    REDIS_SENTINEL_MASTER_NAME = "";

    # Debug mode
    BITNAMI_DEBUG = "false";
  };

  # Variables supporting _FILE suffix for Docker secrets
  envVarsWithSecrets = [
    "REDIS_PASSWORD"
    "REDIS_MASTER_PASSWORD"
    "REDIS_TLS_KEY_FILE_PASS"
  ];

  # Declarative directory schema
  runtimeDirs = {
    data = {
      path = "/firestream/redis/data";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Redis data directory (AOF and RDB files)";
    };
    conf = {
      path = "/opt/firestream/redis/etc";
      type = "conf";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Redis configuration directory";
    };
    confDefault = {
      path = "/opt/firestream/redis/etc.default";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Default configuration templates";
    };
    mountedConf = {
      path = "/opt/firestream/redis/mounted-etc";
      type = "conf";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "User-provided configuration mount point";
    };
    logs = {
      path = "/opt/firestream/redis/logs";
      type = "logs";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Redis log directory";
    };
    tmp = {
      path = "/opt/firestream/redis/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files and PID";
    };
    volumeDir = {
      path = "/firestream/redis";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Persistent volume root";
    };
    # Redis AOF (Append-Only File) directory for persistence
    appendonlydir = {
      path = "/firestream/redis/data/appendonlydir";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Redis append-only file directory for AOF persistence";
    };
  };

  # Validation function
  validateFn = redisHelpers + ''
    error_code=0
    ${validateScript}
    [[ "$error_code" -eq 0 ]] || exit "$error_code"
  '';

  # Activation: Load secrets
  activateFn = redisHelpers + ''
    info "Activating Redis configuration..."

    # Load secrets from _FILE variables
    ${secretsScript}

    info "Redis configuration activated"
  '';

  # Configuration generation
  configFn = redisHelpers + configScript;

  # Initialization (replication setup)
  initFn = redisHelpers + initScript;

  # Startup command
  runCmd = ''
    ${redisHelpers}

    info "Starting Redis ${redisVersion}..."

    # Build extra flags array (not using 'local' since runCmd runs at script top-level)
    flags=()
    if [[ -n "$REDIS_EXTRA_FLAGS" ]]; then
      read -ra flags <<< "$REDIS_EXTRA_FLAGS"
    fi

    # Run as redis user if running as root
    if [[ "$(id -u)" -eq 0 ]]; then
      exec su-exec redis redis-server "$REDIS_CONF_FILE" "''${flags[@]}"
    else
      exec redis-server "$REDIS_CONF_FILE" "''${flags[@]}"
    fi
  '';

  inherit systemDeps runtimeBinDeps;

  exposedPorts = [ 6379 ];
  volumes = [ "/firestream/redis" ];

  user = { name = "redis"; group = "redis"; uid = 1001; gid = 1001; };

  # Development shell extras
  devShellPackages = with pkgs; [ docker docker-compose ];
  devShellHook = ''
    echo "Redis Version: ${redisVersion}"
    echo "Redis Binary: ${redis}/bin/redis-server"
    echo ""
    echo "Build commands:"
    echo "  nix build .#dockerImage    - Build the Docker image"
    echo "  docker load < result       - Load image into Docker"
  '';
}
