# PostgreSQL validation logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

info "Validating settings in POSTGRESQL_* env vars..."

# Password validation
if is_boolean_yes "$ALLOW_EMPTY_PASSWORD"; then
  warn "You set ALLOW_EMPTY_PASSWORD=yes. This is not recommended for production."
else
  if is_empty_value "$POSTGRESQL_PASSWORD"; then
    print_validation_error "POSTGRESQL_PASSWORD is empty. Set ALLOW_EMPTY_PASSWORD=yes to allow empty passwords."
  fi
  if [[ ${#POSTGRESQL_PASSWORD} -gt 100 ]]; then
    print_validation_error "Password cannot be longer than 100 characters."
  fi
fi

# Port validation
if ! validate_port "$POSTGRESQL_PORT_NUMBER" 2>/dev/null; then
  print_validation_error "Invalid POSTGRESQL_PORT_NUMBER: $POSTGRESQL_PORT_NUMBER"
fi

# Replication validation
if [[ -n "$POSTGRESQL_REPLICATION_MODE" ]]; then
  case "$POSTGRESQL_REPLICATION_MODE" in
    master)
      if [[ ${POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS:-0} -lt 0 ]]; then
        print_validation_error "POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS cannot be less than 0."
      fi
      ;;
    slave)
      if is_empty_value "$POSTGRESQL_MASTER_HOST"; then
        print_validation_error "POSTGRESQL_MASTER_HOST is required for slave mode."
      fi
      if is_empty_value "$POSTGRESQL_REPLICATION_USER"; then
        print_validation_error "POSTGRESQL_REPLICATION_USER is required for slave mode."
      fi
      ;;
    *)
      print_validation_error "Invalid replication mode: $POSTGRESQL_REPLICATION_MODE. Use 'master' or 'slave'."
      ;;
  esac

  # Replication password check
  if [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && is_empty_value "$POSTGRESQL_REPLICATION_PASSWORD"; then
    if ! is_boolean_yes "$ALLOW_EMPTY_PASSWORD"; then
      print_validation_error "POSTGRESQL_REPLICATION_PASSWORD is required."
    fi
  fi
fi

# TLS validation
if is_boolean_yes "$POSTGRESQL_ENABLE_TLS"; then
  if is_empty_value "$POSTGRESQL_TLS_CERT_FILE"; then
    print_validation_error "POSTGRESQL_TLS_CERT_FILE is required for TLS."
  elif [[ ! -f "$POSTGRESQL_TLS_CERT_FILE" ]]; then
    print_validation_error "TLS certificate file not found: $POSTGRESQL_TLS_CERT_FILE"
  fi
  if is_empty_value "$POSTGRESQL_TLS_KEY_FILE"; then
    print_validation_error "POSTGRESQL_TLS_KEY_FILE is required for TLS."
  elif [[ ! -f "$POSTGRESQL_TLS_KEY_FILE" ]]; then
    print_validation_error "TLS key file not found: $POSTGRESQL_TLS_KEY_FILE"
  fi
fi

# LDAP validation
if is_boolean_yes "$POSTGRESQL_ENABLE_LDAP"; then
  if is_empty_value "$POSTGRESQL_LDAP_URL" && is_empty_value "$POSTGRESQL_LDAP_SERVER"; then
    print_validation_error "Either POSTGRESQL_LDAP_URL or POSTGRESQL_LDAP_SERVER is required for LDAP."
  fi
fi

# Yes/No validation for boolean fields
for var in ALLOW_EMPTY_PASSWORD POSTGRESQL_ALLOW_REMOTE_CONNECTIONS POSTGRESQL_ENABLE_TLS \
           POSTGRESQL_ENABLE_LDAP POSTGRESQL_REPLICATION_USE_PASSFILE \
           POSTGRESQL_TLS_PREFER_SERVER_CIPHERS; do
  val="${!var:-}"
  if [[ -n "$val" ]]; then
    case "${val,,}" in
      yes|no|true|false|1|0) ;;
      *) print_validation_error "Invalid value for $var: $val (expected yes/no)" ;;
    esac
  fi
done

# Synchronous commit mode validation
if [[ -n "$POSTGRESQL_SYNCHRONOUS_COMMIT_MODE" ]]; then
  case "$POSTGRESQL_SYNCHRONOUS_COMMIT_MODE" in
    on|off|local|remote_write|remote_apply) ;;
    *) print_validation_error "Invalid POSTGRESQL_SYNCHRONOUS_COMMIT_MODE: $POSTGRESQL_SYNCHRONOUS_COMMIT_MODE" ;;
  esac
fi

# Shutdown mode validation
if [[ -n "$POSTGRESQL_SHUTDOWN_MODE" ]]; then
  case "$POSTGRESQL_SHUTDOWN_MODE" in
    smart|fast|immediate) ;;
    *) print_validation_error "Invalid POSTGRESQL_SHUTDOWN_MODE: $POSTGRESQL_SHUTDOWN_MODE" ;;
  esac
fi

info "Validation complete."
