#!/bin/bash
# Copyright Firestream. MIT License.
# SPDX-License-Identifier: APACHE-2.0
#
# Validation script for Apache Spark container
# Validates all configuration before starting Spark.
#
# This script is sourced by the entrypoint and uses functions from
# the Firestream core libraries (log, validations, etc.)

########################
# Validate settings in SPARK_* environment variables
# Globals:
#   SPARK_*
# Arguments:
#   None
# Returns:
#   0 if valid, 1 if errors found
#########################
spark_validate() {
    local error_code=0

    # Auxiliary function to print errors and set error flag
    print_validation_error() {
        error "$1"
        error_code=1
    }

    info "Validating Spark configuration..."

    # ============================================================
    # Validate Spark Mode
    # ============================================================
    case "$SPARK_MODE" in
        master|worker|driver|executor)
            debug "Spark mode validated: $SPARK_MODE"
            ;;
        *)
            print_validation_error "Invalid SPARK_MODE '$SPARK_MODE'. Supported values: master, worker, driver, executor"
            ;;
    esac

    # ============================================================
    # Validate Mode-Specific Requirements
    # ============================================================

    # Worker mode requires master URL
    if [[ "$SPARK_MODE" == "worker" ]]; then
        if [[ -z "$SPARK_MASTER_URL" ]]; then
            print_validation_error "SPARK_MASTER_URL is required for worker mode"
        else
            debug "Worker will connect to: $SPARK_MASTER_URL"
        fi
    fi

    # Kubernetes driver mode requirements
    if [[ "$SPARK_MODE" == "driver" ]]; then
        if [[ -z "${SPARK_DRIVER_BIND_ADDRESS:-}" ]]; then
            warn "SPARK_DRIVER_BIND_ADDRESS not set, will use default network interface"
        fi
    fi

    # Kubernetes executor mode requirements
    if [[ "$SPARK_MODE" == "executor" ]]; then
        if [[ -z "${SPARK_DRIVER_URL:-}" ]]; then
            print_validation_error "SPARK_DRIVER_URL is required for executor mode"
        fi
        if [[ -z "${SPARK_EXECUTOR_ID:-}" ]]; then
            print_validation_error "SPARK_EXECUTOR_ID is required for executor mode"
        fi
        if [[ -z "${SPARK_EXECUTOR_CORES:-}" ]]; then
            print_validation_error "SPARK_EXECUTOR_CORES is required for executor mode"
        fi
        if [[ -z "${SPARK_APPLICATION_ID:-}" ]]; then
            print_validation_error "SPARK_APPLICATION_ID is required for executor mode"
        fi
        if [[ -z "${SPARK_EXECUTOR_POD_IP:-}" ]]; then
            print_validation_error "SPARK_EXECUTOR_POD_IP is required for executor mode"
        fi
        if [[ -z "${SPARK_EXECUTOR_POD_NAME:-}" ]]; then
            warn "SPARK_EXECUTOR_POD_NAME not set, will use default"
        fi
    fi

    # ============================================================
    # Validate Boolean Variables
    # ============================================================

    # Metrics enabled validation
    if ! is_true_false_value "${SPARK_METRICS_ENABLED:-false}"; then
        print_validation_error "SPARK_METRICS_ENABLED must be 'true' or 'false', got: $SPARK_METRICS_ENABLED"
    fi

    # SSL enabled validation
    if ! is_boolean_yes_no "${SPARK_SSL_ENABLED:-no}"; then
        print_validation_error "SPARK_SSL_ENABLED must be 'yes' or 'no', got: $SPARK_SSL_ENABLED"
    fi

    # RPC authentication validation
    if ! is_boolean_yes_no "${SPARK_RPC_AUTHENTICATION_ENABLED:-no}"; then
        print_validation_error "SPARK_RPC_AUTHENTICATION_ENABLED must be 'yes' or 'no', got: $SPARK_RPC_AUTHENTICATION_ENABLED"
    fi

    # RPC encryption validation
    if ! is_boolean_yes_no "${SPARK_RPC_ENCRYPTION_ENABLED:-no}"; then
        print_validation_error "SPARK_RPC_ENCRYPTION_ENABLED must be 'yes' or 'no', got: $SPARK_RPC_ENCRYPTION_ENABLED"
    fi

    # Local storage encryption validation
    if ! is_boolean_yes_no "${SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED:-no}"; then
        print_validation_error "SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED must be 'yes' or 'no', got: $SPARK_LOCAL_STORAGE_ENCRYPTION_ENABLED"
    fi

    # ============================================================
    # Validate SSL Configuration
    # ============================================================
    if is_boolean_yes "${SPARK_SSL_ENABLED:-no}"; then
        info "Validating SSL configuration..."

        if [[ -z "${SPARK_SSL_KEY_PASSWORD:-}" ]]; then
            print_validation_error "SPARK_SSL_KEY_PASSWORD is required when SSL is enabled"
        fi

        if [[ -z "${SPARK_SSL_KEYSTORE_PASSWORD:-}" ]]; then
            print_validation_error "SPARK_SSL_KEYSTORE_PASSWORD is required when SSL is enabled"
        fi

        if [[ -z "${SPARK_SSL_TRUSTSTORE_PASSWORD:-}" ]]; then
            print_validation_error "SPARK_SSL_TRUSTSTORE_PASSWORD is required when SSL is enabled"
        fi

        if [[ ! -f "${SPARK_SSL_KEYSTORE_FILE:-}" ]]; then
            print_validation_error "Keystore file not found: ${SPARK_SSL_KEYSTORE_FILE:-not set}"
        fi

        if [[ ! -f "${SPARK_SSL_TRUSTSTORE_FILE:-}" ]]; then
            print_validation_error "Truststore file not found: ${SPARK_SSL_TRUSTSTORE_FILE:-not set}"
        fi
    fi

    # ============================================================
    # Validate RPC Authentication
    # ============================================================
    if is_boolean_yes "${SPARK_RPC_AUTHENTICATION_ENABLED:-no}"; then
        info "Validating RPC authentication configuration..."

        if [[ -z "${SPARK_RPC_AUTHENTICATION_SECRET:-}" ]]; then
            print_validation_error "SPARK_RPC_AUTHENTICATION_SECRET is required when RPC authentication is enabled"
        fi

        # RPC encryption requires authentication
        if is_boolean_yes "${SPARK_RPC_ENCRYPTION_ENABLED:-no}"; then
            debug "RPC encryption enabled (requires authentication)"
        fi
    else
        # RPC encryption without authentication is not allowed
        if is_boolean_yes "${SPARK_RPC_ENCRYPTION_ENABLED:-no}"; then
            print_validation_error "SPARK_RPC_AUTHENTICATION_ENABLED must be enabled when SPARK_RPC_ENCRYPTION_ENABLED is enabled"
        fi
    fi

    # ============================================================
    # Validate Java Home
    # ============================================================
    if [[ ! -d "${JAVA_HOME:-}" ]]; then
        warn "JAVA_HOME not set or not a directory: ${JAVA_HOME:-not set}"
    elif [[ ! -x "${JAVA_HOME}/bin/java" ]]; then
        print_validation_error "Java executable not found at ${JAVA_HOME}/bin/java"
    else
        local java_version
        java_version=$("${JAVA_HOME}/bin/java" -version 2>&1 | head -1)
        debug "Java version: $java_version"
    fi

    # ============================================================
    # Final Result
    # ============================================================
    if [[ "$error_code" -eq 0 ]]; then
        info "Spark configuration validated successfully"
    else
        error "Spark configuration validation failed"
    fi

    return "$error_code"
}

########################
# Helper: Check if value is true/false (string)
# Arguments:
#   $1 - value to check
# Returns:
#   0 if true/false, 1 otherwise
#########################
is_true_false_value() {
    local value="${1:-}"
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
# Helper: Check if value is yes/no (Bitnami pattern)
# Arguments:
#   $1 - value to check
# Returns:
#   0 if yes/no, 1 otherwise
#########################
is_boolean_yes_no() {
    local value="${1:-}"
    case "$value" in
        yes|no|YES|NO|Yes|No|true|false|TRUE|FALSE|True|False|1|0)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

########################
# Helper: Check if value is boolean yes (Bitnami pattern)
# Arguments:
#   $1 - value to check
# Returns:
#   0 if yes/true/1, 1 otherwise
#########################
is_boolean_yes() {
    local value="${1:-}"
    case "$value" in
        yes|YES|Yes|true|TRUE|True|1)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}
