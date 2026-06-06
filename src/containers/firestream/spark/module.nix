# Spark Container Module - Using Firestream Java Factory
# Copyright Firestream. MIT License.
#
# This module defines the Apache Spark container using mkJavaContainerModule.
# The Java factory provides automatic JDK configuration, JVM tuning options,
# NSS wrapper support for Kubernetes, and classpath management.
#
# Usage:
#   sparkModule = import ./module.nix {
#     inherit pkgs lib firestream sparkVersion;
#   };

{ pkgs
, lib
, firestream
# Canonical version arg (matches eval-container's java runtime contract).
# `sparkVersion` is derived below so the body logic is unchanged.
, version ? "4.0.0"
, jdk ? pkgs.temurin-bin-17
, python ? pkgs.python312

# Externalized core-surface config. Defaults below are EXACTLY today's literals
# so the legacy flake.nix path (which does not pass these) and evalContainer
# (which passes the same values from options.nix) yield identical factory args.

# Paths configuration
, paths ? {
    base = "/opt/spark";
    conf = "/opt/spark/conf";
    data = "/firestream/spark/data";
    logs = "/opt/spark/logs";
  }

# Environment variables with defaults
, envVars ? {
    # Paths (Bitnami compatibility)
    BITNAMI_ROOT_DIR = "/opt/bitnami";
    BITNAMI_VOLUME_DIR = "/bitnami";

    # Firestream paths
    FIRESTREAM_ROOT_DIR = "/opt/firestream";
    FIRESTREAM_VOLUME_DIR = "/firestream";

    # Spark paths
    SPARK_HOME = "/opt/spark";
    SPARK_BASE_DIR = "/opt/spark";
    SPARK_CONF_DIR = "/opt/spark/conf";
    SPARK_DEFAULT_CONF_DIR = "/opt/spark/conf.default";
    SPARK_CONF_FILE = "/opt/spark/conf/spark-defaults.conf";
    SPARK_WORK_DIR = "/opt/spark/work";
    SPARK_LOG_DIR = "/opt/spark/logs";
    SPARK_TMP_DIR = "/opt/spark/tmp";
    SPARK_JARS_DIR = "/opt/spark/jars";
    SPARK_USER_JARS_DIR = "/firestream/spark/jars";
    SPARK_DATA_DIR = "/firestream/spark/data";
    SPARK_INITSCRIPTS_DIR = "/docker-entrypoint-initdb.d";

    # Spark mode
    SPARK_MODE = "master";
    SPARK_MASTER_URL = "spark://spark-master:7077";
    SPARK_NO_DAEMONIZE = "true";

    # User configuration
    SPARK_USER = "spark";
    SPARK_DAEMON_USER = "spark";
    SPARK_DAEMON_GROUP = "spark";

    # Security defaults (disabled)
    SPARK_RPC_AUTHENTICATION_ENABLED = "no";
    SPARK_RPC_ENCRYPTION_ENABLED = "no";
    SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED = "no";
    SPARK_SSL_ENABLED = "no";
    SPARK_SSL_NEED_CLIENT_AUTH = "yes";
    SPARK_SSL_PROTOCOL = "TLSv1.2";
    SPARK_METRICS_ENABLED = "false";

    # Note: JAVA_HOME is auto-set by mkJavaContainerModule

    # Python configuration
    PYTHONPATH = "/opt/spark/python:/opt/spark/python/lib/py4j-0.10.9.7-src.zip";
    PYSPARK_PYTHON = "${python}/bin/python3";
    PYSPARK_DRIVER_PYTHON = "${python}/bin/python3";

    # Debug mode
    BITNAMI_DEBUG = "false";
    MODULE = "spark";
  }

# Variables that support Docker secrets (_FILE suffix)
, envVarsWithSecrets ? [
    "SPARK_MODE"
    "SPARK_MASTER_URL"
    "SPARK_RPC_AUTHENTICATION_ENABLED"
    "SPARK_RPC_AUTHENTICATION_SECRET"
    "SPARK_RPC_ENCRYPTION_ENABLED"
    "SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED"
    "SPARK_SSL_ENABLED"
    "SPARK_SSL_KEY_PASSWORD"
    "SPARK_SSL_KEYSTORE_PASSWORD"
    "SPARK_SSL_KEYSTORE_FILE"
    "SPARK_SSL_TRUSTSTORE_PASSWORD"
    "SPARK_SSL_TRUSTSTORE_FILE"
    "SPARK_SSL_NEED_CLIENT_AUTH"
    "SPARK_SSL_PROTOCOL"
    "SPARK_WEBUI_SSL_PORT"
    "SPARK_METRICS_ENABLED"
  ]

, exposedPorts ? [
    7077   # Spark master
    8080   # Spark master UI
    8081   # Spark worker UI
    4040   # Spark application UI
    6066   # Spark REST submission port
  ]

# In-image health/SBOM service configuration (Phase 4). Forwarded to
# mkJavaContainerModule (which forwards to mkContainerModule). Default-off
# preserves byte-identical legacy-flake behaviour.
, health ? { enable = false; port = 9180; readinessCmd = null; }

# Image naming passthrough (parity defaults).
, imageName ? "firestream-spark"
, imageTag ? version
}:

let
  # Alias to keep the body logic identical to the legacy sparkVersion-based code.
  sparkVersion = version;

  # Read external script files
  validateScript = builtins.readFile ./scripts/validate.sh;
  configScript = builtins.readFile ./scripts/config.sh;
  initScript = builtins.readFile ./scripts/init.sh;
  secretsScript = builtins.readFile ./scripts/secrets.sh;

  # Spark-specific helper functions (used by multiple scripts)
  sparkHelpers = ''
    ########################
    # Check if value is boolean yes (Bitnami pattern)
    # Arguments:
    #   $1 - value to check
    # Returns:
    #   0 if yes/true/1, 1 otherwise
    #########################
    is_boolean_yes() {
        local value="''${1:-}"
        case "$value" in
            yes|YES|Yes|true|TRUE|True|1)
                return 0
                ;;
            *)
                return 1
                ;;
        esac
    }

    ########################
    # Check if value is true/false
    # Arguments:
    #   $1 - value to check
    # Returns:
    #   0 if true/false, 1 otherwise
    #########################
    is_true_false_value() {
        local value="''${1:-}"
        case "$value" in
            true|false|TRUE|FALSE|True|False)
                return 0
                ;;
            *)
                return 1
                ;;
        esac
    }

    ########################
    # Check if running as root
    # Returns:
    #   0 if root, 1 otherwise
    #########################
    am_i_root() {
        [[ "$(id -u)" -eq 0 ]]
    }
  '';

  # System dependencies (Spark-specific extras)
  # Note: Java deps (jdk, cacert, openssl, zlib, procps, etc.) are auto-included
  # by mkJavaContainerModule
  systemDeps = with pkgs; [
    # Python (for PySpark)
    python
    # Networking
    curl wget netcat-gnu
    # Terminal
    ncurses readline
  ];

  # Runtime binary deps (need to be in PATH)
  # Note: jdk is auto-included by mkJavaContainerModule
  runtimeBinDeps = with pkgs; [
    coreutils bash gnused gnugrep gawk findutils which
    curl netcat-gnu procps python
  ];

  # Spark configuration template
  # Placeholders are replaced at runtime via activateFn
  sparkDefaultsTemplate = ''
    # Spark Configuration
    # Generated by Firestream Spark container
    # Placeholders are replaced at container startup

    # Core settings
    spark.master                     {{SPARK_MASTER_URL}}
    spark.submit.deployMode          client

    # UI settings
    spark.ui.enabled                 true

    # Logging
    spark.eventLog.enabled           false
  '';

in firestream.mkJavaContainerModule {
  name = "spark";
  version = sparkVersion;

  # Java configuration - use passed-in JDK
  inherit jdk;

  # JVM tuning for Spark workloads
  heapOpts = "-Xmx1g -Xms512m";
  gcOpts = "-XX:+UseG1GC -XX:MaxGCPauseMillis=20 -XX:InitiatingHeapOccupancyPercent=35";

  # Classpath: Spark JAR directories
  jarDirs = [ "/opt/spark/jars" "/firestream/spark/jars" ];

  # Paths, environment variables, and secret-aware variables are externalized
  # as function arguments (defaults equal to the historical literals). The
  # legacy flake.nix path uses the defaults; evalContainer passes the same
  # values from options.nix, yielding identical factory args.
  inherit paths envVars envVarsWithSecrets;

  # Image naming passthrough.
  inherit imageName imageTag;

  # User configuration (UID 1001 for Bitnami compatibility)
  user = {
    name = "spark";
    group = "spark";
    uid = 1001;
    gid = 1001;
  };

  # Runtime directories with declarative schema
  runtimeDirs = {
    home = {
      path = "/opt/spark";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark installation directory";
    };
    conf = {
      path = "/opt/spark/conf";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark configuration files";
    };
    confDefault = {
      path = "/opt/spark/conf.default";
      type = "conf";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Default configuration templates";
    };
    work = {
      path = "/opt/spark/work";
      type = "work";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark executor work directory";
    };
    logs = {
      path = "/opt/spark/logs";
      type = "logs";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark application and daemon logs";
    };
    tmp = {
      path = "/opt/spark/tmp";
      type = "tmp";
      persistence = "ephemeral";
      mode = "1777";
      description = "Temporary files";
    };
    jars = {
      path = "/opt/spark/jars";
      type = "data";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark JAR files";
    };
    userJars = {
      path = "/firestream/spark/jars";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "User-provided JAR files";
    };
    data = {
      path = "/firestream/spark/data";
      type = "data";
      persistence = "persistent";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Spark data directory";
    };
    initScripts = {
      path = "/docker-entrypoint-initdb.d";
      type = "custom";
      persistence = "ephemeral";
      mode = "0755";
      owner = 1001;
      group = 1001;
      description = "Custom initialization scripts";
    };
    state = {
      path = "/firestream/spark/.state";
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
    "/opt/spark/conf/spark-defaults.conf.template" = sparkDefaultsTemplate;
  };

  # Per-container helpers: emitted at top-level of libhelpersspark.sh by the
  # engine, so chart init containers can `source /opt/bitnami/scripts/libspark.sh`
  # and use these helpers directly.
  perContainerHelpers = sparkHelpers;

  # Validation function (runs first)
  validateFn = secretsScript + validateScript + ''
    # Load secrets from files first
    spark_load_secrets

    # Run validation
    spark_validate
  '';

  # Activation: Replace {{PLACEHOLDERS}} with runtime values
  activateFn = ''
    info "Activating Spark configuration..."

    # Process config template if exists. The template is baked into the image
    # at /opt/spark/conf/spark-defaults.conf.template (read-only), but the
    # output conf_file must live at SPARK_CONF_FILE so K8s-injected env
    # (Bitnami chart sets /opt/bitnami/spark/conf) wins over the baked default.
    # Without this, charts that mount an emptyDir at /opt/bitnami/spark/conf
    # and enforce readOnlyRootFilesystem would fail with
    # "/opt/spark/conf/spark-defaults.conf: Read-only file system".
    local template_file="/opt/spark/conf/spark-defaults.conf.template"
    local conf_file="''${SPARK_CONF_FILE:-/opt/spark/conf/spark-defaults.conf}"

    if [[ -f "$template_file" ]] && [[ ! -f "$conf_file" ]]; then
      ${pkgs.gnused}/bin/sed \
        -e "s|{{SPARK_MASTER_URL}}|''${SPARK_MASTER_URL:-spark://localhost:7077}|g" \
        "$template_file" > "$conf_file"

      info "Generated spark-defaults.conf from template"
    fi

    # Save config hash for change detection. save_config_hash itself is
    # tolerant of read-only state dirs (Bitnami chart pods); no extra wrapper
    # needed here.
    if [[ -f "$conf_file" ]]; then
      save_config_hash "spark" "$conf_file"
    fi

    info "Spark configuration activated"
  '';

  # Initialization (directory setup, first-run config)
  initFn = configScript + initScript + ''
    # Run initialization
    spark_initialize

    # Run custom init scripts
    spark_custom_init_scripts
  '';

  # Runtime config adjustments
  configFn = configScript;

  # Startup command based on mode
  runCmd = ''
    ${sparkHelpers}

    info "Starting Spark in ''${SPARK_MODE:-master} mode..."

    # The env-defaults set SPARK_HOME=/opt/spark for bitnami compatibility, but
    # /opt/spark only holds writable runtime dirs (conf/, jars/, logs/, work/).
    # The spark-class wrapper sources $SPARK_HOME/bin/load-spark-env.sh, which
    # only exists under the nix-store spark package — override SPARK_HOME to
    # point there before exec.
    export SPARK_HOME=${pkgs.spark}

    case "''${SPARK_MODE:-master}" in
      master)
        info "Starting Spark Master..."
        exec ${pkgs.spark}/bin/spark-class org.apache.spark.deploy.master.Master
        ;;
      worker)
        info "Starting Spark Worker connecting to ''${SPARK_MASTER_URL}..."
        # Pass --work-dir explicitly so the Worker writes to a writable path.
        # Bitnami chart sets SPARK_WORK_DIR=/opt/bitnami/spark/work (emptyDir);
        # without --work-dir, spark defaults to $SPARK_HOME/work which is
        # /nix/store/...spark.../work (read-only).
        exec ${pkgs.spark}/bin/spark-class org.apache.spark.deploy.worker.Worker \
          --work-dir "''${SPARK_WORK_DIR:-/opt/spark/work}" \
          "''${SPARK_MASTER_URL}"
        ;;
      driver)
        # Kubernetes driver mode
        # Arguments passed via environment
        info "Starting Spark Driver for Kubernetes..."
        exec ${pkgs.spark}/bin/spark-submit \
          --conf "spark.driver.bindAddress=''${SPARK_DRIVER_BIND_ADDRESS:-}" \
          --conf "spark.executorEnv.SPARK_DRIVER_POD_IP=''${SPARK_DRIVER_BIND_ADDRESS:-}" \
          --conf "spark.jars.ivy=/tmp/.ivy" \
          --deploy-mode client \
          "$@"
        ;;
      executor)
        # Kubernetes executor mode
        info "Starting Spark Executor for Kubernetes..."

        # Parse SPARK_JAVA_OPT_* variables
        local java_opts=()
        while IFS='=' read -r key value; do
          java_opts+=("$value")
        done < <(env | grep "^SPARK_JAVA_OPT_" | sort -t_ -k4 -n | sed 's/[^=]*=//')

        exec ${jdk}/bin/java \
          "''${java_opts[@]}" \
          "-Xms''${SPARK_EXECUTOR_MEMORY:-1g}" \
          "-Xmx''${SPARK_EXECUTOR_MEMORY:-1g}" \
          -cp '/opt/spark/conf::/opt/spark/jars/*' \
          org.apache.spark.scheduler.cluster.k8s.KubernetesExecutorBackend \
          --driver-url "''${SPARK_DRIVER_URL}" \
          --executor-id "''${SPARK_EXECUTOR_ID}" \
          --cores "''${SPARK_EXECUTOR_CORES}" \
          --app-id "''${SPARK_APPLICATION_ID}" \
          --hostname "''${SPARK_EXECUTOR_POD_IP}" \
          --resourceProfileId "''${SPARK_RESOURCE_PROFILE_ID:-0}" \
          --podName "''${SPARK_EXECUTOR_POD_NAME:-}"
        ;;
      *)
        error "Unknown SPARK_MODE: ''${SPARK_MODE}"
        exit 1
        ;;
    esac
  '';

  inherit systemDeps runtimeBinDeps;

  # Exposed ports
  inherit exposedPorts;

  # In-image firestream-healthd (Phase 4).
  inherit health;

  # Volume paths
  volumes = [
    "/opt/spark/work"
    "/opt/spark/logs"
    "/firestream/spark/data"
    "/firestream/spark/jars"
  ];

  # Development shell extras
  devShellPackages = with pkgs; [ docker docker-compose scala sbt ];
  devShellHook = ''
    echo "Spark Version: ${sparkVersion}"
    echo "Java: ${jdk.name}"
    echo "Python: ${python.name}"
    echo ""
    echo "Build commands:"
    echo "  nix build .#dockerImage    - Build the Docker image"
    echo "  docker load < result       - Load image into Docker"
    echo ""
    echo "Spark commands (in dev shell):"
    echo "  spark-shell               - Start Spark Scala shell"
    echo "  pyspark                   - Start PySpark shell"
    echo "  spark-submit              - Submit Spark applications"
  '';
}
