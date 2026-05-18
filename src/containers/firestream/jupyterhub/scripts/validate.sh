# JupyterHub validation logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Validates JUPYTERHUB_* environment variables before initialization

debug "Validating JupyterHub environment variables..."

# Validate yes/no values
for var in JUPYTERHUB_SKIP_BOOTSTRAP; do
    val="${!var:-}"
    if [[ -n "$val" ]]; then
        case "${val,,}" in
            yes|no|true|false|1|0) ;;
            *) print_validation_error "Invalid value for $var: $val (expected yes/no)" ;;
        esac
    fi
done

# Validate port numbers
for port_var in JUPYTERHUB_PROXY_PORT_NUMBER JUPYTERHUB_API_PORT_NUMBER JUPYTERHUB_DATABASE_PORT_NUMBER; do
    port="${!port_var:-}"
    if [[ -n "$port" ]]; then
        if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 ]] || [[ "$port" -gt 65535 ]]; then
            print_validation_error "Invalid port for $port_var: $port"
        fi
    fi
done

# Validate database type
if [[ -n "${JUPYTERHUB_DATABASE_TYPE:-}" ]]; then
    case "$JUPYTERHUB_DATABASE_TYPE" in
        postgresql|sqlite) ;;
        *) print_validation_error "JUPYTERHUB_DATABASE_TYPE must be 'postgresql' or 'sqlite'" ;;
    esac
fi

# Validate database host can be resolved (only if using PostgreSQL)
if [[ "${JUPYTERHUB_DATABASE_TYPE:-postgresql}" == "postgresql" ]]; then
    if [[ -n "${JUPYTERHUB_DATABASE_HOST:-}" ]]; then
        if ! getent hosts "$JUPYTERHUB_DATABASE_HOST" >/dev/null 2>&1; then
            warn "Database host '$JUPYTERHUB_DATABASE_HOST' could not be resolved - this may cause connection issues"
        fi
    fi
fi

# Validate spawner type
if [[ -n "${JUPYTERHUB_SPAWNER:-}" ]]; then
    case "$JUPYTERHUB_SPAWNER" in
        localprocess|docker|kubernetes|simple) ;;
        *) print_validation_error "JUPYTERHUB_SPAWNER must be 'localprocess', 'docker', 'kubernetes', or 'simple'" ;;
    esac
fi

# Validate authenticator type
if [[ -n "${JUPYTERHUB_AUTHENTICATOR:-}" ]]; then
    case "$JUPYTERHUB_AUTHENTICATOR" in
        pam|dummy|oauth|ldap) ;;
        *) print_validation_error "JUPYTERHUB_AUTHENTICATOR must be 'pam', 'dummy', 'oauth', or 'ldap'" ;;
    esac
fi

# Validate credentials unless empty passwords are explicitly allowed
if is_boolean_yes "${ALLOW_EMPTY_PASSWORD:-no}"; then
    warn "ALLOW_EMPTY_PASSWORD is enabled - do not use in production"
else
    if is_empty_value "${JUPYTERHUB_PASSWORD:-}"; then
        print_validation_error "JUPYTERHUB_PASSWORD is required (set ALLOW_EMPTY_PASSWORD=yes to override)"
    fi
    if [[ "${JUPYTERHUB_DATABASE_TYPE:-postgresql}" == "postgresql" ]]; then
        if is_empty_value "${JUPYTERHUB_DATABASE_PASSWORD:-}"; then
            print_validation_error "JUPYTERHUB_DATABASE_PASSWORD is required when using PostgreSQL (set ALLOW_EMPTY_PASSWORD=yes to override)"
        fi
    fi
fi

# Validate username is set
if is_empty_value "${JUPYTERHUB_USERNAME:-}"; then
    warn "JUPYTERHUB_USERNAME is not set - using default 'user'"
fi
