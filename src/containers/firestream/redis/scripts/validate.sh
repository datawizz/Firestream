# Redis Validation Logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly
#
# Validates REDIS_* environment variables before initialization

debug "Validating Redis environment variables..."

# Validate password configuration
if is_boolean_yes "$ALLOW_EMPTY_PASSWORD"; then
    warn "ALLOW_EMPTY_PASSWORD is enabled - do not use in production"
else
    if is_empty_value "${REDIS_PASSWORD:-}"; then
        print_validation_error "REDIS_PASSWORD is required (set ALLOW_EMPTY_PASSWORD=yes to override)"
    fi
fi

# Validate replication mode
if [[ -n "$REDIS_REPLICATION_MODE" ]]; then
    case "$REDIS_REPLICATION_MODE" in
        master|slave|replica)
            ;;
        *)
            print_validation_error "Invalid REDIS_REPLICATION_MODE: $REDIS_REPLICATION_MODE (expected master/replica)"
            ;;
    esac

    # Validate replica-specific settings
    if [[ "$REDIS_REPLICATION_MODE" =~ ^(slave|replica)$ ]]; then
        if [[ -n "$REDIS_MASTER_PORT_NUMBER" ]]; then
            if ! [[ "$REDIS_MASTER_PORT_NUMBER" =~ ^[0-9]+$ ]] || \
               [[ "$REDIS_MASTER_PORT_NUMBER" -lt 1 ]] || \
               [[ "$REDIS_MASTER_PORT_NUMBER" -gt 65535 ]]; then
                print_validation_error "Invalid REDIS_MASTER_PORT_NUMBER: $REDIS_MASTER_PORT_NUMBER"
            fi
        fi
        if ! is_boolean_yes "$ALLOW_EMPTY_PASSWORD" && is_empty_value "${REDIS_MASTER_PASSWORD:-}"; then
            print_validation_error "REDIS_MASTER_PASSWORD is required for replica mode (set ALLOW_EMPTY_PASSWORD=yes to override)"
        fi
    fi
fi

# Validate port numbers
for port_var in REDIS_PORT_NUMBER REDIS_TLS_PORT_NUMBER REDIS_MASTER_PORT_NUMBER REDIS_SENTINEL_PORT_NUMBER; do
    port="${!port_var:-}"
    if [[ -n "$port" ]]; then
        if ! [[ "$port" =~ ^[0-9]+$ ]] || [[ "$port" -lt 1 ]] || [[ "$port" -gt 65535 ]]; then
            print_validation_error "Invalid port for $port_var: $port"
        fi
    fi
done

# Validate TLS configuration
if is_boolean_yes "$REDIS_TLS_ENABLED"; then
    # Check for port conflict
    if [[ "$REDIS_PORT_NUMBER" == "$REDIS_TLS_PORT_NUMBER" ]] && [[ "$REDIS_PORT_NUMBER" != "6379" ]]; then
        print_validation_error "REDIS_PORT_NUMBER and REDIS_TLS_PORT_NUMBER point to the same port ($REDIS_PORT_NUMBER). Change one or disable non-TLS traffic by setting REDIS_PORT_NUMBER=0"
    fi

    # Validate certificate file
    if is_empty_value "${REDIS_TLS_CERT_FILE:-}"; then
        print_validation_error "REDIS_TLS_CERT_FILE is required when TLS is enabled"
    elif [[ ! -f "$REDIS_TLS_CERT_FILE" ]]; then
        print_validation_error "TLS certificate file not found: $REDIS_TLS_CERT_FILE"
    fi

    # Validate key file
    if is_empty_value "${REDIS_TLS_KEY_FILE:-}"; then
        print_validation_error "REDIS_TLS_KEY_FILE is required when TLS is enabled"
    elif [[ ! -f "$REDIS_TLS_KEY_FILE" ]]; then
        print_validation_error "TLS key file not found: $REDIS_TLS_KEY_FILE"
    fi

    # Validate CA file or directory
    if is_empty_value "${REDIS_TLS_CA_FILE:-}" && is_empty_value "${REDIS_TLS_CA_DIR:-}"; then
        print_validation_error "Either REDIS_TLS_CA_FILE or REDIS_TLS_CA_DIR is required when TLS is enabled"
    fi
    if [[ -n "${REDIS_TLS_CA_FILE:-}" ]] && [[ ! -f "$REDIS_TLS_CA_FILE" ]]; then
        print_validation_error "TLS CA certificate file not found: $REDIS_TLS_CA_FILE"
    fi
    if [[ -n "${REDIS_TLS_CA_DIR:-}" ]] && [[ ! -d "$REDIS_TLS_CA_DIR" ]]; then
        print_validation_error "TLS CA certificate directory not found: $REDIS_TLS_CA_DIR"
    fi

    # Validate DH params file if specified
    if [[ -n "${REDIS_TLS_DH_PARAMS_FILE:-}" ]] && [[ ! -f "$REDIS_TLS_DH_PARAMS_FILE" ]]; then
        print_validation_error "TLS DH params file not found: $REDIS_TLS_DH_PARAMS_FILE"
    fi
fi

# Validate yes/no boolean values
for var in REDIS_AOF_ENABLED REDIS_RDB_POLICY_DISABLED REDIS_TLS_ENABLED REDIS_ALLOW_REMOTE_CONNECTIONS ALLOW_EMPTY_PASSWORD; do
    val="${!var:-}"
    if [[ -n "$val" ]]; then
        case "${val,,}" in
            yes|no|true|false|1|0) ;;
            *) print_validation_error "Invalid boolean value for $var: $val (expected yes/no)" ;;
        esac
    fi
done
