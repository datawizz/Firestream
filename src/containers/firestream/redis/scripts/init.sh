# Redis Initialization Logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# Handles replication configuration and first-run initialization

info "Initializing Redis..."

# Create required directories
for dir in "$REDIS_BASE_DIR" "$REDIS_DATA_DIR" "$REDIS_TMP_DIR" "$REDIS_LOG_DIR" "$REDIS_CONF_DIR"; do
    ensure_dir_exists "$dir"
done

# Ensure volume directory exists
ensure_dir_exists "$REDIS_VOLUME_DIR"

# Create AOF directory for persistence (must exist before Redis starts)
# This is needed because the volume mount overwrites build-time directories
ensure_dir_exists "${REDIS_DATA_DIR}/appendonlydir"

# Check for initialization marker
REDIS_INIT_MARKER="${REDIS_DATA_DIR}/.redis_initialized"

if [[ -f "$REDIS_INIT_MARKER" ]]; then
    info "Redis already initialized, restoring state..."
else
    info "First run detected - initializing Redis..."
fi

# Configure replication if mode is set
if [[ -n "$REDIS_REPLICATION_MODE" ]] && [[ ! -e "${REDIS_MOUNTED_CONF_DIR}/redis.conf" ]]; then
    info "Configuring replication mode: $REDIS_REPLICATION_MODE"

    # Set replica announcement IP and port
    redis_conf_set replica-announce-ip "${REDIS_REPLICA_IP:-$(get_machine_ip)}"
    redis_conf_set replica-announce-port "${REDIS_REPLICA_PORT:-$REDIS_MASTER_PORT_NUMBER}"

    # Enable TLS for replication if TLS is enabled
    if is_boolean_yes "$REDIS_TLS_ENABLED"; then
        redis_conf_set tls-replication yes
    fi

    if [[ "$REDIS_REPLICATION_MODE" == "master" ]]; then
        # Master needs masterauth for replica authentication
        if [[ -n "$REDIS_PASSWORD" ]]; then
            redis_conf_set masterauth "$REDIS_PASSWORD"
        fi
    elif [[ "$REDIS_REPLICATION_MODE" =~ ^(slave|replica)$ ]]; then
        # Replica mode - discover master from Sentinel if configured
        if [[ -n "$REDIS_SENTINEL_HOST" ]]; then
            info "Discovering master from Sentinel at $REDIS_SENTINEL_HOST:$REDIS_SENTINEL_PORT_NUMBER..."

            local -a sentinel_cmd=("redis-cli" "-h" "$REDIS_SENTINEL_HOST" "-p" "$REDIS_SENTINEL_PORT_NUMBER")

            if is_boolean_yes "$REDIS_TLS_ENABLED"; then
                sentinel_cmd+=("--tls" "--cert" "$REDIS_TLS_CERT_FILE" "--key" "$REDIS_TLS_KEY_FILE")
                if is_empty_value "$REDIS_TLS_CA_FILE"; then
                    sentinel_cmd+=("--cacertdir" "$REDIS_TLS_CA_DIR")
                else
                    sentinel_cmd+=("--cacert" "$REDIS_TLS_CA_FILE")
                fi
            fi

            sentinel_cmd+=("sentinel" "get-master-addr-by-name" "$REDIS_SENTINEL_MASTER_NAME")

            read -r -a REDIS_SENTINEL_INFO <<< "$("${sentinel_cmd[@]}" | tr '\n' ' ')"
            REDIS_MASTER_HOST="${REDIS_SENTINEL_INFO[0]}"
            REDIS_MASTER_PORT_NUMBER="${REDIS_SENTINEL_INFO[1]}"

            info "Discovered master: $REDIS_MASTER_HOST:$REDIS_MASTER_PORT_NUMBER"
        fi

        # Wait for master to be available
        info "Waiting for master at $REDIS_MASTER_HOST:$REDIS_MASTER_PORT_NUMBER..."
        redis_wait_for_master

        # Configure master authentication
        if [[ -n "$REDIS_MASTER_PASSWORD" ]]; then
            redis_conf_set masterauth "$REDIS_MASTER_PASSWORD"
        fi

        # Configure replication - use 'replicaof' for Redis 5+ (fallback to 'slaveof' for older versions)
        local repl_cmd="replicaof"
        if [[ $(redis_major_version) -lt 5 ]]; then
            repl_cmd="slaveof"
        fi
        redis_conf_set "$repl_cmd" "$REDIS_MASTER_HOST $REDIS_MASTER_PORT_NUMBER"
    fi
fi

# Create initialization marker on first run
if [[ ! -f "$REDIS_INIT_MARKER" ]]; then
    touch "$REDIS_INIT_MARKER"
    info "Initialization marker created"
fi

info "Redis initialization complete"
