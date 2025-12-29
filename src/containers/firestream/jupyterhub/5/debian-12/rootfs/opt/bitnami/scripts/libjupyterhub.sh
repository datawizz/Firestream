#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# Bitnami JupyterHub library
#
# shellcheck disable=SC1091

########################
# Load global libraries
########################
# Note: These are expected to exist in the container from prebuildfs or Nix environment

# Logging functions (inline fallback if liblog.sh not available)
if [[ -f /opt/bitnami/scripts/liblog.sh ]]; then
    . /opt/bitnami/scripts/liblog.sh
else
    info() { echo "[INFO] $*"; }
    warn() { echo "[WARN] $*" >&2; }
    error() { echo "[ERROR] $*" >&2; }
    debug() { [[ "${BITNAMI_DEBUG:-false}" == "true" ]] && echo "[DEBUG] $*"; }
fi

# OS helper functions (inline fallback)
if [[ -f /opt/bitnami/scripts/libos.sh ]]; then
    . /opt/bitnami/scripts/libos.sh
else
    am_i_root() { [[ "$(id -u)" -eq 0 ]]; }
    get_uid() { id -u "${1:-}"; }
    exec_as_user() {
        local user="${1:?user missing}"
        shift
        if am_i_root; then
            run_as_user "$user" "$@"
        else
            exec "$@"
        fi
    }
    run_as_user() {
        local user="${1:?user missing}"
        shift
        su -s /bin/bash -c "$(printf '%q ' "$@")" "$user"
    }
fi

# File system helper functions
if [[ -f /opt/bitnami/scripts/libfs.sh ]]; then
    . /opt/bitnami/scripts/libfs.sh
fi

# Validation helper functions
if [[ -f /opt/bitnami/scripts/libvalidations.sh ]]; then
    . /opt/bitnami/scripts/libvalidations.sh
fi

# Persistence helper functions
if [[ -f /opt/bitnami/scripts/libpersistence.sh ]]; then
    . /opt/bitnami/scripts/libpersistence.sh
fi

# Service helper functions
if [[ -f /opt/bitnami/scripts/libservice.sh ]]; then
    . /opt/bitnami/scripts/libservice.sh
fi

########################
# Helper functions
########################

# Ensure a directory exists
ensure_dir_exists() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir"
    fi
}

# Check if a value is empty
is_empty_value() {
    local value="${1:-}"
    [[ -z "$value" ]]
}

# Check if a value is a boolean yes
is_boolean_yes() {
    local bool="${1:-}"
    local -l lowercased="${bool}"
    [[ "$lowercased" =~ ^(yes|true|1)$ ]]
}

# Check if a value is yes or no
is_yes_no_value() {
    local value="${1:-}"
    local -l lowercased="${value}"
    [[ "$lowercased" =~ ^(yes|no)$ ]]
}

# Check if a value is true or false
is_true_false_value() {
    local value="${1:-}"
    local -l lowercased="${value}"
    [[ "$lowercased" =~ ^(true|false)$ ]]
}

# Print validation error
print_validation_error() {
    error "$1"
    error_code=1
}

# Replace text in a file
replace_in_file() {
    local filename="${1:?filename missing}"
    local pattern="${2:?pattern missing}"
    local replacement="${3:?replacement missing}"
    local delimiter="${4:-/}"
    sed -i "s${delimiter}${pattern}${delimiter}${replacement}${delimiter}g" "$filename"
}

########################
# Welcome page
########################

print_welcome_page() {
    local -r app_name="JupyterHub"
    local -r app_version="${APP_VERSION:-5.3.0}"

    info ""
    info "Welcome to the Firestream ${app_name} container"
    info ""
    info "  ${app_name} version: ${app_version}"
    info "  Built with Nix for reproducibility"
    info ""
}

########################
# Save configuration hash for change detection
########################

save_config_hash() {
    local name="$1"
    local file="$2"
    local state_dir="/firestream/${name}/.state"
    ensure_dir_exists "$state_dir"
    if [[ -f "$file" ]]; then
        md5sum "$file" 2>/dev/null | cut -d' ' -f1 > "${state_dir}/config.hash" || true
    fi
}

########################
# Check if config has changed since last run
########################

config_hash_changed() {
    local name="$1"
    local file="$2"
    local state_dir="/firestream/${name}/.state"
    local hash_file="${state_dir}/config.hash"

    if [[ ! -f "$hash_file" ]]; then
        return 0  # No previous hash, consider it changed
    fi

    local current_hash
    current_hash="$(md5sum "$file" 2>/dev/null | cut -d' ' -f1)"
    local stored_hash
    stored_hash="$(cat "$hash_file" 2>/dev/null)"

    [[ "$current_hash" != "$stored_hash" ]]
}

########################
# Wait for PostgreSQL to be ready
########################

jupyterhub_wait_for_postgresql() {
    local host="${JUPYTERHUB_DATABASE_HOST:-postgresql}"
    local port="${JUPYTERHUB_DATABASE_PORT_NUMBER:-5432}"
    local timeout="${JUPYTERHUB_DB_WAIT_TIMEOUT:-120}"

    info "Waiting for PostgreSQL at $host:$port..."

    if wait-for-port --host "$host" --timeout "$timeout" "$port"; then
        info "PostgreSQL is available"
        return 0
    else
        error "Timeout waiting for PostgreSQL after ${timeout}s"
        return 1
    fi
}

########################
# Generate default JupyterHub configuration
########################

jupyterhub_generate_default_config() {
    local conf_file="${1:?configuration file missing}"

    info "Generating default JupyterHub configuration..."

    cat > "$conf_file" << 'EOFCONFIG'
# JupyterHub configuration file
# Generated by Firestream container initialization

c = get_config()

# Proxy settings
c.JupyterHub.bind_url = 'http://:8000'
c.JupyterHub.hub_bind_url = 'http://:8081'

# Spawner
c.JupyterHub.spawner_class = 'jupyterhub.spawner.LocalProcessSpawner'

# Authenticator
c.JupyterHub.authenticator_class = 'jupyterhub.auth.PAMAuthenticator'

# Data directory
c.JupyterHub.data_files_path = '/opt/bitnami/jupyterhub/data'

# Admin users (set via environment)
import os
admin_user = os.environ.get('JUPYTERHUB_USERNAME', 'user')
c.Authenticator.admin_users = {admin_user}
c.Authenticator.allowed_users = {admin_user}
EOFCONFIG
}

########################
# Validate JupyterHub settings
########################

jupyterhub_validate() {
    debug "Validating JupyterHub environment variables..."
    local error_code=0

    # Load validation script if exists
    if [[ -f /opt/bitnami/scripts/jupyterhub/validate.sh ]]; then
        . /opt/bitnami/scripts/jupyterhub/validate.sh
    fi

    # Check for validation errors
    if [[ "$error_code" -ne 0 ]]; then
        error "JupyterHub validation failed"
        return 1
    fi

    return 0
}

########################
# Initialize JupyterHub
########################

jupyterhub_initialize() {
    local -r app_name="jupyterhub"

    info "Initializing JupyterHub..."

    # Ensure required directories exist
    for dir in "$JUPYTERHUB_BASE_DIR" "$JUPYTERHUB_CONF_DIR" "$JUPYTERHUB_DATA_DIR" \
               "$JUPYTERHUB_LOGS_DIR" "$JUPYTERHUB_TMP_DIR" "$JUPYTERHUB_VOLUME_DIR"; do
        ensure_dir_exists "$dir"
    done

    # Wait for PostgreSQL if using it
    if [[ "${JUPYTERHUB_DATABASE_TYPE:-postgresql}" == "postgresql" ]]; then
        if ! jupyterhub_wait_for_postgresql; then
            return 1
        fi
    fi

    # Check initialization marker
    local init_marker="${JUPYTERHUB_DATA_DIR}/.jupyterhub_initialized"

    if [[ -f "$init_marker" ]]; then
        info "JupyterHub already initialized"

        # Check for config changes
        if config_hash_changed "$app_name" "$JUPYTERHUB_CONF_FILE"; then
            info "Configuration changed since last startup"
        fi
    else
        info "First run - initializing JupyterHub..."

        if ! is_boolean_yes "${JUPYTERHUB_SKIP_BOOTSTRAP:-no}"; then
            # Load configuration script
            if [[ -f /opt/bitnami/scripts/jupyterhub/config.sh ]]; then
                . /opt/bitnami/scripts/jupyterhub/config.sh
            else
                # Generate default configuration
                jupyterhub_generate_default_config "$JUPYTERHUB_CONF_FILE"
            fi

            # Upgrade database schema if using PostgreSQL
            if [[ "${JUPYTERHUB_DATABASE_TYPE:-postgresql}" == "postgresql" ]]; then
                info "Upgrading database schema..."
                jupyterhub upgrade-db --config "${JUPYTERHUB_CONF_FILE}" 2>/dev/null || \
                    warn "Database upgrade returned non-zero (may be normal for fresh DB)"
            fi

            # Generate cookie secret if not exists
            local cookie_secret_file="${JUPYTERHUB_DATA_DIR}/jupyterhub_cookie_secret"
            if [[ ! -f "$cookie_secret_file" ]]; then
                info "Generating cookie secret..."
                openssl rand -hex 32 > "$cookie_secret_file"
                chmod 600 "$cookie_secret_file"
            fi

            # Generate proxy auth token if not exists
            local proxy_auth_file="${JUPYTERHUB_DATA_DIR}/proxy_auth_token"
            if [[ ! -f "$proxy_auth_file" ]]; then
                info "Generating proxy auth token..."
                openssl rand -hex 32 > "$proxy_auth_file"
                chmod 600 "$proxy_auth_file"
            fi

            info "JupyterHub bootstrap complete"
        else
            info "JUPYTERHUB_SKIP_BOOTSTRAP is set - skipping bootstrap"
        fi

        # Create initialization marker
        touch "$init_marker"
    fi

    # Save current config hash
    save_config_hash "$app_name" "$JUPYTERHUB_CONF_FILE"

    return 0
}

########################
# Check if JupyterHub is running
########################

is_jupyterhub_running() {
    local pid
    pid="$(cat "${JUPYTERHUB_PID_FILE:-/opt/bitnami/jupyterhub/tmp/jupyterhub.pid}" 2>/dev/null)"

    if [[ -n "$pid" ]]; then
        kill -0 "$pid" 2>/dev/null
    else
        return 1
    fi
}

########################
# Check if JupyterHub is not running
########################

is_jupyterhub_not_running() {
    ! is_jupyterhub_running
}

########################
# Stop JupyterHub
########################

jupyterhub_stop() {
    is_jupyterhub_not_running && return

    local pid
    pid="$(cat "${JUPYTERHUB_PID_FILE:-/opt/bitnami/jupyterhub/tmp/jupyterhub.pid}" 2>/dev/null)"

    if [[ -n "$pid" ]]; then
        info "Stopping JupyterHub (PID: $pid)..."
        kill "$pid" 2>/dev/null || true

        # Wait for graceful shutdown
        local timeout=30
        local elapsed=0
        while kill -0 "$pid" 2>/dev/null && [[ $elapsed -lt $timeout ]]; do
            sleep 1
            ((elapsed++))
        done

        # Force kill if still running
        if kill -0 "$pid" 2>/dev/null; then
            warn "JupyterHub did not stop gracefully, forcing..."
            kill -9 "$pid" 2>/dev/null || true
        fi
    fi
}

########################
# Add or modify JupyterHub configuration
########################

jupyterhub_conf_set() {
    local key="${1:?key missing}"
    local value="${2:?value missing}"
    local conf_file="${JUPYTERHUB_CONF_FILE}"

    debug "Setting ${key} = '${value}' in JupyterHub configuration"

    # Check if key already exists
    if grep -q "^c\.${key}" "$conf_file" 2>/dev/null; then
        # Replace existing line
        sed -i "s|^c\.${key}.*|c.${key} = '${value}'|" "$conf_file"
    else
        # Append new line
        echo "c.${key} = '${value}'" >> "$conf_file"
    fi
}

########################
# Get JupyterHub configuration value
########################

jupyterhub_conf_get() {
    local key="${1:?key missing}"
    local conf_file="${JUPYTERHUB_CONF_FILE}"

    grep "^c\.${key}" "$conf_file" 2>/dev/null | sed "s|^c\.${key} = ||" | tr -d "\"'"
}
