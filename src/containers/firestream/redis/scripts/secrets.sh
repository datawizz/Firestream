# Redis Docker Secrets Loading
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Loads environment variables from Docker secrets (_FILE pattern)
# For each VAR_FILE, reads the file content into VAR

debug "Loading secrets from _FILE environment variables..."

# List of environment variables that support Docker secrets
redis_secret_vars=(
    REDIS_PASSWORD
    REDIS_MASTER_PASSWORD
    REDIS_TLS_KEY_FILE_PASS
)

for env_var in "${redis_secret_vars[@]}"; do
    file_env_var="${env_var}_FILE"
    if [[ -n "${!file_env_var:-}" ]]; then
        if [[ -r "${!file_env_var:-}" ]]; then
            debug "Loading ${env_var} from ${!file_env_var}"
            export "${env_var}=$(< "${!file_env_var}")"
            unset "${file_env_var}"
        else
            warn "Skipping export of '${env_var}'. '${!file_env_var:-}' is not readable."
        fi
    fi
done

unset redis_secret_vars
