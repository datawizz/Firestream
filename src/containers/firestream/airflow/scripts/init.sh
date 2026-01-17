# Airflow initialization logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

# Helper function for retry_while - checks if admin user exists
# Must handle set -e properly by not letting the command trigger immediate exit
is_airflow_admin_created() {
    local airflow_users
    airflow_users="$(airflow users list --output plain 2>&1 | grep -v DEBUG)" || true
    if echo "${airflow_users}" | grep -q "$AIRFLOW_USERNAME"; then
        return 0
    fi
    return 1
}

# Helper function for retry_while - checks if DB migrations are complete
# Must handle set -e properly by not letting the command trigger immediate exit
is_db_migrated() {
    local result=0
    airflow db check-migrations --migration-wait-timeout=0 >/dev/null 2>&1 || result=$?
    return $result
}

info "Initializing Airflow (component: $AIRFLOW_COMPONENT_TYPE)..."

# Create directories with proper permissions
for dir in "$AIRFLOW_HOME" "$AIRFLOW_DAGS_DIR" "$AIRFLOW_LOGS_DIR" \
           "$AIRFLOW_SCHEDULER_LOGS_DIR" "$AIRFLOW_PLUGINS_DIR" "$AIRFLOW_TMP_DIR"; do
  ensure_dir_exists "$dir"
done

# Generate configuration if needed
if [[ ! -f "$AIRFLOW_CONF_FILE" ]] || is_boolean_yes "${AIRFLOW_FORCE_OVERWRITE_CONF_FILE:-no}"; then
  airflow_configure
else
  info "Using existing configuration file"
fi

# Wait for database
info "Waiting for database at ${AIRFLOW_DATABASE_HOST}:${AIRFLOW_DATABASE_PORT_NUMBER}..."
wait-for-port --host "$AIRFLOW_DATABASE_HOST" --timeout 120 "$AIRFLOW_DATABASE_PORT_NUMBER"

# Database operations
major_version=$(airflow_major_version)
db_init_cmd="migrate"
[[ $major_version -eq 2 ]] && db_init_cmd="init"

case "$AIRFLOW_COMPONENT_TYPE" in
  webserver|api-server)
    # Remove stale PID file
    rm -f "${AIRFLOW_TMP_DIR}/airflow-${AIRFLOW_COMPONENT_TYPE}.pid"

    if is_boolean_yes "$AIRFLOW_SKIP_DB_SETUP"; then
      info "Skipping DB setup, waiting for migrations..."
      retry_while "airflow db check-migrations --migration-wait-timeout=$AIRFLOW_DB_MIGRATE_TIMEOUT" --tries 30 --sleep 10
    elif ! airflow db check-migrations --migration-wait-timeout=0 2>&1 | grep -v DEBUG; then
      info "Initializing database..."
      airflow db $db_init_cmd

      # Create admin user
      if [[ -n "$AIRFLOW_USERNAME" && -n "$AIRFLOW_PASSWORD" ]]; then
        info "Creating admin user..."
        if ! airflow users create \
          -r "Admin" \
          -u "$AIRFLOW_USERNAME" \
          -e "$AIRFLOW_EMAIL" \
          -p "$AIRFLOW_PASSWORD" \
          -f "$AIRFLOW_FIRSTNAME" \
          -l "$AIRFLOW_LASTNAME" 2>&1; then
          error "Failed to create admin user"
          exit 1
        fi
        # Verify user was created
        info "Verifying admin user was created..."
        if airflow users list --output plain 2>&1 | grep -v DEBUG | grep -q "$AIRFLOW_USERNAME"; then
          info "Admin user '$AIRFLOW_USERNAME' created successfully"
        else
          error "Admin user verification failed"
          exit 1
        fi
      fi

      # Create pool if specified
      if [[ -n "${AIRFLOW_POOL_NAME:-}" && -n "${AIRFLOW_POOL_SIZE:-}" && -n "${AIRFLOW_POOL_DESC:-}" ]]; then
        info "Creating pool: $AIRFLOW_POOL_NAME"
        if [[ $major_version -eq 2 ]]; then
          if ! airflow pool -s "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" 2>&1; then
            warn "Pool creation failed, continuing..."
          fi
        else
          if ! airflow pools set "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" 2>&1; then
            warn "Pool creation failed, continuing..."
          fi
        fi
      fi

      info "Synchronizing permissions..."
      if ! airflow sync-perm --include-dags 2>&1; then
        warn "Permission sync failed, continuing..."
      fi
    else
      info "Database already initialized, running migrations..."
      airflow db migrate

      # Check if admin exists
      if ! airflow users list --output plain 2>&1 | grep -v DEBUG | grep -q "$AIRFLOW_USERNAME"; then
        info "Creating admin user..."
        if ! airflow users create \
          -r "Admin" \
          -u "$AIRFLOW_USERNAME" \
          -e "$AIRFLOW_EMAIL" \
          -p "$AIRFLOW_PASSWORD" \
          -f "$AIRFLOW_FIRSTNAME" \
          -l "$AIRFLOW_LASTNAME" 2>&1; then
          error "Failed to create admin user"
          exit 1
        fi
        # Verify user was created
        info "Verifying admin user was created..."
        if airflow users list --output plain 2>&1 | grep -v DEBUG | grep -q "$AIRFLOW_USERNAME"; then
          info "Admin user '$AIRFLOW_USERNAME' created successfully"
        else
          error "Admin user verification failed"
          exit 1
        fi
      else
        info "Admin user '$AIRFLOW_USERNAME' already exists"
      fi

      if ! airflow sync-perm --include-dags 2>&1; then
        warn "Permission sync failed, continuing..."
      fi
    fi
    ;;

  *)
    # Workers, schedulers, triggerers wait for webserver to init
    info "Waiting for database migrations..."
    retry_while --tries 60 --sleep 10 is_db_migrated

    info "Waiting for admin user..."
    retry_while --tries 60 --sleep 5 is_airflow_admin_created

    # Celery components wait for Redis
    if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
      info "Waiting for Redis at ${REDIS_HOST}:${REDIS_PORT_NUMBER}..."
      wait-for-port --host "$REDIS_HOST" --timeout 60 "$REDIS_PORT_NUMBER"
    fi
    ;;
esac

info "Initialization complete"
