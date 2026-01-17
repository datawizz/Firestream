#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# JupyterHub initialization logic
#
# This file is sourced, not executed directly.
# Handles database initialization, admin user setup, and first-run configuration.

info "Initializing JupyterHub..."

# Create required directories
for dir in "$JUPYTERHUB_BASE_DIR" "$JUPYTERHUB_CONF_DIR" "$JUPYTERHUB_DATA_DIR" \
           "$JUPYTERHUB_LOGS_DIR" "$JUPYTERHUB_TMP_DIR"; do
    ensure_dir_exists "$dir"
done

# Ensure volume directory exists and has correct permissions
ensure_dir_exists "$JUPYTERHUB_VOLUME_DIR"

# Wait for PostgreSQL if configured
if [[ "${JUPYTERHUB_DATABASE_TYPE:-sqlite}" == "postgresql" ]]; then
    info "Waiting for PostgreSQL at ${JUPYTERHUB_DATABASE_HOST}:${JUPYTERHUB_DATABASE_PORT_NUMBER}..."
    if ! jupyterhub_wait_for_postgresql; then
        error "Failed to connect to PostgreSQL"
        exit 1
    fi
fi

# Check if this is first run (no initialization marker)
JUPYTERHUB_INIT_MARKER="${JUPYTERHUB_DATA_DIR}/.jupyterhub_initialized"

if [[ -f "$JUPYTERHUB_INIT_MARKER" ]]; then
    info "JupyterHub already initialized, restoring persisted state..."

    # Check for config changes
    if config_hash_changed "jupyterhub" "$JUPYTERHUB_CONF_FILE"; then
        info "Configuration changed since last startup"
    fi

else
    info "First run detected - initializing JupyterHub..."

    if ! is_boolean_yes "$JUPYTERHUB_SKIP_BOOTSTRAP"; then
        # Upgrade database schema if using PostgreSQL
        if [[ "${JUPYTERHUB_DATABASE_TYPE:-sqlite}" == "postgresql" ]]; then
            info "Upgrading database schema..."
            if ! jupyterhub upgrade-db --config "${JUPYTERHUB_CONF_FILE}" 2>/dev/null; then
                warn "Database upgrade returned non-zero exit code, this may be normal for fresh databases"
            fi
        fi

        # Create admin user entry in database
        # Note: JupyterHub creates users on first login, but we configure admin users in the config
        if [[ -n "${JUPYTERHUB_USERNAME:-}" ]]; then
            info "Admin user configured: ${JUPYTERHUB_USERNAME}"
            # For PAM authenticator, the system user must exist
            if [[ "${JUPYTERHUB_AUTHENTICATOR:-pam}" == "pam" ]]; then
                if ! id "${JUPYTERHUB_USERNAME}" >/dev/null 2>&1; then
                    info "Creating system user for PAM authentication: ${JUPYTERHUB_USERNAME}"
                    # This requires root privileges - skip if not root
                    if am_i_root; then
                        useradd -m -s /bin/bash "${JUPYTERHUB_USERNAME}" 2>/dev/null || true
                        if [[ -n "${JUPYTERHUB_PASSWORD:-}" ]]; then
                            echo "${JUPYTERHUB_USERNAME}:${JUPYTERHUB_PASSWORD}" | chpasswd 2>/dev/null || true
                        fi
                    else
                        warn "Cannot create PAM user - not running as root"
                    fi
                fi
            fi
        fi

        # Generate cookie secret if not exists
        cookie_secret_file="${JUPYTERHUB_DATA_DIR}/jupyterhub_cookie_secret"
        if [[ ! -f "$cookie_secret_file" ]]; then
            info "Generating cookie secret..."
            openssl rand -hex 32 > "$cookie_secret_file"
            chmod 600 "$cookie_secret_file"
        fi

        # Generate proxy auth token if not exists
        proxy_auth_file="${JUPYTERHUB_DATA_DIR}/proxy_auth_token"
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
    touch "$JUPYTERHUB_INIT_MARKER"
    info "Initialization marker created"
fi

info "JupyterHub initialization complete"
