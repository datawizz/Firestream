# Odoo secrets loading
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Loads secrets from _FILE environment variables (Docker secrets pattern)

# List of variables that support _FILE suffix
odoo_secret_vars=(
    "ODOO_PASSWORD"
    "ODOO_DATABASE_PASSWORD"
    "ODOO_SMTP_PASSWORD"
)

for env_var in "${odoo_secret_vars[@]}"; do
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
    "POSTGRESQL_HOST:ODOO_DATABASE_HOST"
    "POSTGRESQL_PORT_NUMBER:ODOO_DATABASE_PORT_NUMBER"
    "POSTGRESQL_DATABASE_NAME:ODOO_DATABASE_NAME"
    "POSTGRESQL_DATABASE_USER:ODOO_DATABASE_USER"
    "POSTGRESQL_DATABASE_USERNAME:ODOO_DATABASE_USER"
    "POSTGRESQL_DATABASE_PASSWORD:ODOO_DATABASE_PASSWORD"
    "SMTP_HOST:ODOO_SMTP_HOST"
    "SMTP_PORT:ODOO_SMTP_PORT_NUMBER"
    "SMTP_USER:ODOO_SMTP_USER"
    "SMTP_PASSWORD:ODOO_SMTP_PASSWORD"
    "SMTP_PROTOCOL:ODOO_SMTP_PROTOCOL"
)

for mapping in "${legacy_vars[@]}"; do
    legacy_var="${mapping%%:*}"
    target_var="${mapping##*:}"

    if [[ -n "${!legacy_var:-}" ]] && [[ -z "${!target_var:-}" ]]; then
        debug "Using legacy variable ${legacy_var} for ${target_var}"
        export "${target_var}=${!legacy_var}"
    fi
done

unset odoo_secret_vars legacy_vars
