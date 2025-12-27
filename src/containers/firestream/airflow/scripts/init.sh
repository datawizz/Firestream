# Airflow initialization logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

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
wait_for_port "$AIRFLOW_DATABASE_HOST" "$AIRFLOW_DATABASE_PORT_NUMBER" --tries 24 --sleep 5

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
    elif ! airflow db check-migrations 2>/dev/null; then
      info "Initializing database..."
      airflow db $db_init_cmd

      # Create admin user
      if [[ -n "$AIRFLOW_USERNAME" && -n "$AIRFLOW_PASSWORD" ]]; then
        info "Creating admin user..."
        airflow users create \
          -r "Admin" \
          -u "$AIRFLOW_USERNAME" \
          -e "$AIRFLOW_EMAIL" \
          -p "$AIRFLOW_PASSWORD" \
          -f "$AIRFLOW_FIRSTNAME" \
          -l "$AIRFLOW_LASTNAME" || true
      fi

      # Create pool if specified
      if [[ -n "${AIRFLOW_POOL_NAME:-}" && -n "${AIRFLOW_POOL_SIZE:-}" && -n "${AIRFLOW_POOL_DESC:-}" ]]; then
        info "Creating pool: $AIRFLOW_POOL_NAME"
        if [[ $major_version -eq 2 ]]; then
          airflow pool -s "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" || true
        else
          airflow pools set "$AIRFLOW_POOL_NAME" "$AIRFLOW_POOL_SIZE" "$AIRFLOW_POOL_DESC" || true
        fi
      fi

      info "Synchronizing permissions..."
      airflow sync-perm --include-dags || true
    else
      info "Database already initialized, running migrations..."
      airflow db migrate

      # Check if admin exists
      if ! airflow users list --output plain 2>/dev/null | grep -q "$AIRFLOW_USERNAME"; then
        info "Creating admin user..."
        airflow users create \
          -r "Admin" \
          -u "$AIRFLOW_USERNAME" \
          -e "$AIRFLOW_EMAIL" \
          -p "$AIRFLOW_PASSWORD" \
          -f "$AIRFLOW_FIRSTNAME" \
          -l "$AIRFLOW_LASTNAME" || true
      fi

      airflow sync-perm --include-dags || true
    fi
    ;;

  *)
    # Workers, schedulers, triggerers wait for webserver to init
    info "Waiting for database migrations..."
    retry_while "airflow db check-migrations --migration-wait-timeout=$AIRFLOW_DB_MIGRATE_TIMEOUT" --tries 60 --sleep 10

    info "Waiting for admin user..."
    retry_while "airflow users list --output plain 2>/dev/null | grep -q '$AIRFLOW_USERNAME'" --tries 60 --sleep 5

    # Celery components wait for Redis
    if [[ "$AIRFLOW_EXECUTOR" == "CeleryExecutor" || "$AIRFLOW_EXECUTOR" == "CeleryKubernetesExecutor" ]]; then
      info "Waiting for Redis at ${REDIS_HOST}:${REDIS_PORT_NUMBER}..."
      wait_for_port "$REDIS_HOST" "$REDIS_PORT_NUMBER" --tries 12 --sleep 5
    fi
    ;;
esac

info "Initialization complete"
