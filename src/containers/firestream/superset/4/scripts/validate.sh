# Superset validation logic
# This file is sourced, not executed directly
# Copyright Firestream. MIT License.

# Validate SUPERSET_ROLE
case "${SUPERSET_ROLE:-webserver}" in
  webserver|celery-worker|celery-beat|celery-flower|init)
    debug "SUPERSET_ROLE is valid: ${SUPERSET_ROLE}"
    ;;
  *)
    print_validation_error "Invalid SUPERSET_ROLE: ${SUPERSET_ROLE}"
    print_validation_error "Allowed values: webserver, celery-worker, celery-beat, celery-flower, init"
    ;;
esac

# Validate secret key (critical for security)
if is_empty_value "${SUPERSET_SECRET_KEY:-}"; then
  warn "SUPERSET_SECRET_KEY not set - will auto-generate"
  warn "This is NOT recommended for production. Set SUPERSET_SECRET_KEY explicitly."
fi

# Validate database configuration
if is_empty_value "${SUPERSET_DATABASE_HOST:-}"; then
  print_validation_error "SUPERSET_DATABASE_HOST is required"
fi

# Validate port numbers
! is_empty_value "${SUPERSET_WEBSERVER_PORT_NUMBER:-}" && {
  if ! [[ "${SUPERSET_WEBSERVER_PORT_NUMBER}" =~ ^[0-9]+$ ]] || \
     [[ "${SUPERSET_WEBSERVER_PORT_NUMBER}" -lt 1 ]] || \
     [[ "${SUPERSET_WEBSERVER_PORT_NUMBER}" -gt 65535 ]]; then
    print_validation_error "SUPERSET_WEBSERVER_PORT_NUMBER must be a valid port (1-65535)"
  fi
}

! is_empty_value "${SUPERSET_DATABASE_PORT_NUMBER:-}" && {
  if ! [[ "${SUPERSET_DATABASE_PORT_NUMBER}" =~ ^[0-9]+$ ]] || \
     [[ "${SUPERSET_DATABASE_PORT_NUMBER}" -lt 1 ]] || \
     [[ "${SUPERSET_DATABASE_PORT_NUMBER}" -gt 65535 ]]; then
    print_validation_error "SUPERSET_DATABASE_PORT_NUMBER must be a valid port (1-65535)"
  fi
}

! is_empty_value "${REDIS_PORT_NUMBER:-}" && {
  if ! [[ "${REDIS_PORT_NUMBER}" =~ ^[0-9]+$ ]] || \
     [[ "${REDIS_PORT_NUMBER}" -lt 1 ]] || \
     [[ "${REDIS_PORT_NUMBER}" -gt 65535 ]]; then
    print_validation_error "REDIS_PORT_NUMBER must be a valid port (1-65535)"
  fi
}

# Validate Redis for Celery roles
case "${SUPERSET_ROLE:-webserver}" in
  celery-worker|celery-beat|celery-flower)
    if is_empty_value "${REDIS_HOST:-}"; then
      print_validation_error "REDIS_HOST is required for Celery roles (${SUPERSET_ROLE})"
    fi
    ;;
esac

# Warn about Flower authentication
if [[ "${SUPERSET_ROLE:-}" == "celery-flower" ]] && is_empty_value "${FLOWER_BASIC_AUTH:-}"; then
  warn "FLOWER_BASIC_AUTH not set - Flower UI will be unauthenticated"
  warn "Set FLOWER_BASIC_AUTH=user:password for production use"
fi

# Process secret key early (needed for config generation)
process_superset_secret_key

debug "Validation complete"
