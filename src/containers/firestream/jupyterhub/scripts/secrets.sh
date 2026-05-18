# JupyterHub secrets loading
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Loads secrets from _FILE environment variables (Docker secrets pattern)

# List of variables that support _FILE suffix
jupyterhub_secret_vars=(
    "JUPYTERHUB_PASSWORD"
    "JUPYTERHUB_DATABASE_PASSWORD"
)

for env_var in "${jupyterhub_secret_vars[@]}"; do
    file_env_var="${env_var}_FILE"
    if [[ -n "${!file_env_var:-}" ]]; then
        if [[ -r "${!file_env_var:-}" ]]; then
            debug "Loading ${env_var} from file ${!file_env_var}"
            export "${env_var}=$(< "${!file_env_var}")"
            unset "${file_env_var}"
        else
            warn "Skipping ${env_var}: file '${!file_env_var:-}' is not readable"
        fi
    fi
done

# Also support legacy PostgreSQL variable names
legacy_vars=(
    "POSTGRESQL_HOST:JUPYTERHUB_DATABASE_HOST"
    "POSTGRESQL_PORT_NUMBER:JUPYTERHUB_DATABASE_PORT_NUMBER"
    "POSTGRESQL_DATABASE_NAME:JUPYTERHUB_DATABASE_NAME"
    "POSTGRESQL_DATABASE_USER:JUPYTERHUB_DATABASE_USER"
    "POSTGRESQL_DATABASE_USERNAME:JUPYTERHUB_DATABASE_USER"
    "POSTGRESQL_DATABASE_PASSWORD:JUPYTERHUB_DATABASE_PASSWORD"
    "POSTGRESQL_DATABASE:JUPYTERHUB_DATABASE_NAME"
    "POSTGRESQL_USERNAME:JUPYTERHUB_DATABASE_USER"
    "POSTGRESQL_PASSWORD:JUPYTERHUB_DATABASE_PASSWORD"
)

for mapping in "${legacy_vars[@]}"; do
    legacy_var="${mapping%%:*}"
    target_var="${mapping##*:}"

    if [[ -n "${!legacy_var:-}" ]] && [[ -z "${!target_var:-}" ]]; then
        debug "Using legacy variable ${legacy_var} for ${target_var}"
        export "${target_var}=${!legacy_var}"
    fi
done

unset jupyterhub_secret_vars legacy_vars
