#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# JupyterHub validation logic
#
# This file is sourced, not executed directly.
# Validates JUPYTERHUB_* environment variables before initialization.

debug "Validating JupyterHub environment variables..."

# Track validation errors
error_code=0

# Validate yes/no values
for var in JUPYTERHUB_SKIP_BOOTSTRAP ALLOW_EMPTY_PASSWORD; do
    val="${!var:-}"
    if [[ -n "$val" ]]; then
        case "${val,,}" in
            yes|no|true|false|1|0) ;;
            *) print_validation_error "Invalid value for $var: $val (expected yes/no/true/false)" ;;
        esac
    fi
done

# Validate port numbers
for port_var in JUPYTERHUB_PROXY_PORT_NUMBER JUPYTERHUB_API_PORT_NUMBER JUPYTERHUB_DATABASE_PORT_NUMBER; do
    port="${!port_var:-}"
    if [[ -n "$port" ]]; then
        if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 ]] || [[ "$port" -gt 65535 ]]; then
            print_validation_error "Invalid port for $port_var: $port (must be 1-65535)"
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

# Validate OAuth configuration if using OAuth authenticator
if [[ "${JUPYTERHUB_AUTHENTICATOR:-}" == "oauth" ]]; then
    if is_empty_value "${JUPYTERHUB_OAUTH_CLIENT_ID:-}"; then
        print_validation_error "JUPYTERHUB_OAUTH_CLIENT_ID is required when using OAuth authenticator"
    fi
    if is_empty_value "${JUPYTERHUB_OAUTH_CLIENT_SECRET:-}"; then
        print_validation_error "JUPYTERHUB_OAUTH_CLIENT_SECRET is required when using OAuth authenticator"
    fi
fi

# Validate LDAP configuration if using LDAP authenticator
if [[ "${JUPYTERHUB_AUTHENTICATOR:-}" == "ldap" ]]; then
    if is_empty_value "${JUPYTERHUB_LDAP_SERVER:-}"; then
        print_validation_error "JUPYTERHUB_LDAP_SERVER is required when using LDAP authenticator"
    fi
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

# Exit if validation errors occurred
if [[ "$error_code" -ne 0 ]]; then
    error "Environment validation failed"
    exit 1
fi
