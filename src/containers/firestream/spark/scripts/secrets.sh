#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# SPDX-License-Identifier: APACHE-2.0
#
# Docker Secrets loader for Apache Spark container
# Implements the Bitnami _FILE pattern for loading secrets from files.
#
# This allows secrets to be mounted as files (e.g., from Kubernetes Secrets)
# and loaded into environment variables at runtime.
#
# Pattern: If VARNAME_FILE is set to a path, read the file contents
#          and export them as VARNAME

########################
# Load secrets from files
# For each variable in the list, if VARNAME_FILE exists,
# load the file contents into VARNAME
# Globals:
#   Various SPARK_* variables
# Arguments:
#   None
# Returns:
#   None
#########################
spark_load_secrets() {
    debug "Loading secrets from files..."

    # List of variables that support the _FILE pattern
    local spark_secret_vars=(
        # Spark mode and connection
        SPARK_MODE
        SPARK_MASTER_URL

        # RPC Authentication
        SPARK_RPC_AUTHENTICATION_ENABLED
        SPARK_RPC_AUTHENTICATION_SECRET
        SPARK_RPC_ENCRYPTION_ENABLED

        # Local Storage Encryption
        SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED

        # SSL/TLS Configuration
        SPARK_SSL_ENABLED
        SPARK_SSL_KEY_PASSWORD
        SPARK_SSL_KEYSTORE_PASSWORD
        SPARK_SSL_KEYSTORE_FILE
        SPARK_SSL_TRUSTSTORE_PASSWORD
        SPARK_SSL_TRUSTSTORE_FILE
        SPARK_SSL_NEED_CLIENT_AUTH
        SPARK_SSL_PROTOCOL
        SPARK_WEBUI_SSL_PORT

        # Metrics
        SPARK_METRICS_ENABLED
    )

    local env_var file_env_var file_path

    for env_var in "${spark_secret_vars[@]}"; do
        file_env_var="${env_var}_FILE"

        # Check if the _FILE variable is set
        if [[ -n "${!file_env_var:-}" ]]; then
            file_path="${!file_env_var}"

            if [[ -r "$file_path" ]]; then
                # Read the file and export the variable
                export "${env_var}=$(< "$file_path")"
                # Unset the _FILE variable (Bitnami pattern)
                unset "${file_env_var}"
                debug "Loaded $env_var from file: $file_path"
            else
                warn "Skipping export of '$env_var': file '$file_path' is not readable"
            fi
        fi
    done

    debug "Secrets loading complete"
}

########################
# Generate a secure random secret
# Arguments:
#   $1 - length (default: 32)
# Returns:
#   Random base64-encoded string
#########################
spark_generate_secret() {
    local length="${1:-32}"
    head -c "$length" /dev/urandom | base64 | tr -dc 'a-zA-Z0-9' | head -c "$length"
}

########################
# Auto-generate RPC authentication secret if not set
# Globals:
#   SPARK_RPC_AUTHENTICATION_SECRET
#   SPARK_RPC_AUTHENTICATION_ENABLED
# Arguments:
#   None
# Returns:
#   None
#########################
spark_ensure_rpc_secret() {
    if is_boolean_yes "${SPARK_RPC_AUTHENTICATION_ENABLED:-no}"; then
        if [[ -z "${SPARK_RPC_AUTHENTICATION_SECRET:-}" ]]; then
            warn "SPARK_RPC_AUTHENTICATION_SECRET not set, generating random secret"
            export SPARK_RPC_AUTHENTICATION_SECRET="$(spark_generate_secret 32)"
            info "Generated RPC authentication secret"
        fi
    fi
}
