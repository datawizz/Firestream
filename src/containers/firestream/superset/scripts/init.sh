# Superset initialization logic
# This file is sourced, not executed directly
# Copyright Firestream. Apache-2.0 License.

info "Initializing Superset (role: ${SUPERSET_ROLE:-webserver})..."

# Create required directories
for dir in "${SUPERSET_HOME:-/opt/superset/superset_home}" \
           "${SUPERSET_LOG_DIR:-/opt/superset/logs}" \
           "${SUPERSET_TMP_DIR:-/opt/superset/tmp}" \
           "/app/pythonpath"; do
  ensure_dir_exists "$dir"
done

# Wait for database unless explicitly skipped
if ! is_boolean_yes "${SUPERSET_SKIP_DATABASE_WAIT:-no}"; then
  local db_host="${SUPERSET_DATABASE_HOST:-postgresql}"
  local db_port="${SUPERSET_DATABASE_PORT_NUMBER:-5432}"

  info "Waiting for database at ${db_host}:${db_port}..."
  if ! wait_for_port "$db_host" "$db_port" --tries 30 --sleep 5; then
    error "Database at ${db_host}:${db_port} is not available"
    exit 1
  fi
  info "Database is available"
fi

# Role-specific initialization
case "${SUPERSET_ROLE:-webserver}" in
  webserver|init)
    # Run database migrations and create admin user
    superset_run_init
    ;;

  celery-worker|celery-beat|celery-flower)
    # Wait for webserver/init to complete database setup
    info "Waiting for database migrations to be applied..."

    local max_tries=60
    local tries=0
    while [[ $tries -lt $max_tries ]]; do
      # Check if migrations have been applied by trying to access FAB tables
      if superset fab list-users &>/dev/null; then
        info "Database is initialized"
        break
      fi
      tries=$((tries + 1))
      debug "Waiting for database initialization... (attempt $tries/$max_tries)"
      sleep 5
    done

    if [[ $tries -ge $max_tries ]]; then
      error "Database initialization timed out. Ensure the init container has run."
      exit 1
    fi

    # Wait for Redis
    if [[ -n "${REDIS_HOST:-}" ]]; then
      local redis_host="${REDIS_HOST}"
      local redis_port="${REDIS_PORT_NUMBER:-6379}"

      info "Waiting for Redis at ${redis_host}:${redis_port}..."
      if ! wait_for_port "$redis_host" "$redis_port" --tries 12 --sleep 5; then
        error "Redis at ${redis_host}:${redis_port} is not available"
        exit 1
      fi
      info "Redis is available"
    fi
    ;;
esac

info "Initialization complete"
