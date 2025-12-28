# Redis Configuration Generation
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly
#
# Generates redis.conf from environment variables

info "Configuring Redis..."

# Remove stale PID file from previous runs
rm -f "$REDIS_PID_FILE"

# Configure directories and permissions
redis_configure_permissions

# Check for user-provided configuration
if [[ -e "${REDIS_MOUNTED_CONF_DIR}/redis.conf" ]]; then
    info "Using mounted configuration file"
    if [[ -e "${REDIS_DEFAULT_CONF_DIR}/redis.conf" ]]; then
        rm -f "${REDIS_DEFAULT_CONF_DIR}/redis.conf"
    fi
    cp "${REDIS_MOUNTED_CONF_DIR}/redis.conf" "${REDIS_CONF_FILE}"
else
    info "Generating Redis configuration from environment variables"

    # Allow remote connections
    if is_boolean_yes "$ALLOW_EMPTY_PASSWORD"; then
        redis_conf_set protected-mode no
    fi
    if is_boolean_yes "$REDIS_ALLOW_REMOTE_CONNECTIONS"; then
        redis_conf_set bind "0.0.0.0 ::"
    fi

    # Configure persistence: AOF
    redis_conf_set appendonly "$REDIS_AOF_ENABLED"

    # Configure persistence: RDB
    if is_empty_value "$REDIS_RDB_POLICY"; then
        if is_boolean_yes "$REDIS_RDB_POLICY_DISABLED"; then
            redis_conf_set save ""
        fi
    else
        # RDB_POLICY format: "seconds#changes seconds#changes ..."
        for policy in ${REDIS_RDB_POLICY}; do
            redis_conf_set save "${policy//#/ }"
        done
    fi

    # Configure port
    redis_conf_set port "$REDIS_PORT_NUMBER"

    # Configure TLS
    if is_boolean_yes "$REDIS_TLS_ENABLED"; then
        if [[ "$REDIS_PORT_NUMBER" == "6379" ]] && [[ "$REDIS_TLS_PORT_NUMBER" == "6379" ]]; then
            # Both ports are default - enable TLS only
            redis_conf_set port 0
            redis_conf_set tls-port "$REDIS_TLS_PORT_NUMBER"
        else
            redis_conf_set tls-port "$REDIS_TLS_PORT_NUMBER"
        fi

        redis_conf_set tls-cert-file "$REDIS_TLS_CERT_FILE"
        redis_conf_set tls-key-file "$REDIS_TLS_KEY_FILE"

        if is_empty_value "$REDIS_TLS_CA_FILE"; then
            redis_conf_set tls-ca-cert-dir "$REDIS_TLS_CA_DIR"
        else
            redis_conf_set tls-ca-cert-file "$REDIS_TLS_CA_FILE"
        fi

        if [[ -n "$REDIS_TLS_KEY_FILE_PASS" ]]; then
            redis_conf_set tls-key-file-pass "$REDIS_TLS_KEY_FILE_PASS"
        fi
        if [[ -n "$REDIS_TLS_DH_PARAMS_FILE" ]]; then
            redis_conf_set tls-dh-params-file "$REDIS_TLS_DH_PARAMS_FILE"
        fi

        redis_conf_set tls-auth-clients "$REDIS_TLS_AUTH_CLIENTS"
    fi

    # Configure multi-threading
    if [[ -n "$REDIS_IO_THREADS_DO_READS" ]]; then
        redis_conf_set io-threads-do-reads "$REDIS_IO_THREADS_DO_READS"
    fi
    if [[ -n "$REDIS_IO_THREADS" ]]; then
        redis_conf_set io-threads "$REDIS_IO_THREADS"
    fi

    # Configure authentication
    if [[ -n "$REDIS_PASSWORD" ]]; then
        redis_conf_set requirepass "$REDIS_PASSWORD"
    else
        redis_conf_unset requirepass
    fi

    # Disable unsafe commands
    if [[ -n "$REDIS_DISABLE_COMMANDS" ]]; then
        redis_disable_unsafe_commands
    fi

    # Configure ACL file
    if [[ -n "$REDIS_ACLFILE" ]]; then
        redis_conf_set aclfile "$REDIS_ACLFILE"
    fi

    # Append include directives for overrides
    redis_append_include_conf
fi

info "Redis configuration complete"
