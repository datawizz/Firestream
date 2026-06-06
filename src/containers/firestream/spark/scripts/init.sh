#!/bin/bash
# Copyright Firestream. MIT License.
# SPDX-License-Identifier: APACHE-2.0
#
# Initialization script for Apache Spark container
# Runs on first container start to set up directories,
# configuration, and run custom init scripts.

########################
# Initialize Spark environment
# Creates necessary directories and generates initial configuration
# Globals:
#   SPARK_*
# Arguments:
#   None
# Returns:
#   None
#########################
spark_initialize() {
    info "Initializing Apache Spark..."

    # ============================================================
    # Create Runtime Directories
    # ============================================================
    info "Creating runtime directories..."

    # Work directory (for executors)
    ensure_dir_exists "$SPARK_WORK_DIR"
    if am_i_root; then
        chown "$SPARK_DAEMON_USER:$SPARK_DAEMON_GROUP" "$SPARK_WORK_DIR"
    fi

    # Log directory
    ensure_dir_exists "$SPARK_LOG_DIR"
    if am_i_root; then
        chown "$SPARK_DAEMON_USER:$SPARK_DAEMON_GROUP" "$SPARK_LOG_DIR"
    fi

    # Temp directory
    ensure_dir_exists "$SPARK_TMP_DIR"
    # chmod may fail on K8s emptyDirs mounted at non-baked paths (e.g. Bitnami
    # chart's /opt/bitnami/spark/tmp owned by root, pod runs as 1001) — best-effort
    chmod 1777 "$SPARK_TMP_DIR" 2>/dev/null || debug "Skipped chmod on $SPARK_TMP_DIR"

    # User jars directory (persistent volume)
    if [[ -n "${SPARK_USER_JARS_DIR:-}" ]]; then
        ensure_dir_exists "$SPARK_USER_JARS_DIR"
        if am_i_root; then
            chown "$SPARK_DAEMON_USER:$SPARK_DAEMON_GROUP" "$SPARK_USER_JARS_DIR"
        fi
    fi

    # Data directory (persistent volume)
    if [[ -n "${SPARK_DATA_DIR:-}" ]]; then
        ensure_dir_exists "$SPARK_DATA_DIR"
        if am_i_root; then
            chown "$SPARK_DAEMON_USER:$SPARK_DAEMON_GROUP" "$SPARK_DATA_DIR"
        fi
    fi

    # ============================================================
    # Configuration Setup
    # ============================================================

    # Copy default configs if not present (don't overwrite user-mounted configs)
    if [[ -d "${SPARK_DEFAULT_CONF_DIR:-}" ]] && [[ -d "${SPARK_CONF_DIR:-}" ]]; then
        debug "Copying default configuration files..."
        cp -nr "${SPARK_DEFAULT_CONF_DIR}/"* "${SPARK_CONF_DIR}/" 2>/dev/null || true
    fi

    # Generate configuration if no spark-defaults.conf exists
    if [[ ! -f "${SPARK_CONF_FILE:-}" ]]; then
        info "No existing configuration found, generating defaults..."
        spark_configure
    else
        info "Using existing configuration file: ${SPARK_CONF_FILE}"
    fi

    # ============================================================
    # Kubernetes-Specific Initialization
    # ============================================================
    case "${SPARK_MODE:-master}" in
        driver)
            info "Initializing Spark driver for Kubernetes..."
            # Driver-specific setup
            if [[ -n "${SPARK_DRIVER_BIND_ADDRESS:-}" ]]; then
                debug "Driver bind address: ${SPARK_DRIVER_BIND_ADDRESS}"
            fi
            ;;
        executor)
            info "Initializing Spark executor for Kubernetes..."
            # Executor-specific setup
            debug "Executor ID: ${SPARK_EXECUTOR_ID:-}"
            debug "Driver URL: ${SPARK_DRIVER_URL:-}"
            debug "Application ID: ${SPARK_APPLICATION_ID:-}"
            ;;
        *)
            # Standard master/worker mode - no special init needed
            ;;
    esac

    info "Spark initialization complete"
}

########################
# Run custom initialization scripts
# Looks for *.sh files in SPARK_INITSCRIPTS_DIR
# Globals:
#   SPARK_INITSCRIPTS_DIR
# Arguments:
#   None
# Returns:
#   None
#########################
spark_custom_init_scripts() {
    local initscripts_dir="${SPARK_INITSCRIPTS_DIR:-/docker-entrypoint-initdb.d}"

    if [[ ! -d "$initscripts_dir" ]]; then
        debug "Init scripts directory does not exist: $initscripts_dir"
        return 0
    fi

    # Find all .sh files
    local script_files
    script_files=$(find "$initscripts_dir" -type f -name "*.sh" 2>/dev/null | sort)

    if [[ -z "$script_files" ]]; then
        debug "No custom init scripts found in $initscripts_dir"
        return 0
    fi

    info "Running custom initialization scripts from $initscripts_dir..."

    local f
    while IFS= read -r f; do
        if [[ -x "$f" ]]; then
            info "Executing: $f"
            "$f"
        else
            info "Sourcing: $f"
            # shellcheck disable=SC1090
            . "$f"
        fi
    done <<< "$script_files"

    info "Custom initialization scripts completed"
}

########################
# Ensure a directory exists with proper permissions
# Arguments:
#   $1 - directory path
# Returns:
#   None
#########################
ensure_dir_exists() {
    local dir="${1:?directory path required}"

    if [[ ! -d "$dir" ]]; then
        debug "Creating directory: $dir"
        mkdir -p "$dir"
    fi
}

########################
# Check if running as root
# Returns:
#   0 if root, 1 otherwise
#########################
am_i_root() {
    [[ "$(id -u)" -eq 0 ]]
}
