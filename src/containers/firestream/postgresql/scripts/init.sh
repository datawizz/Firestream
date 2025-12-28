# PostgreSQL initialization logic
# Copyright Firestream. Apache-2.0 License.
# This file is sourced, not executed directly

info "Initializing PostgreSQL..."

########################
# Execute SQL statements
# Arguments:
#   $1 - database (optional)
#   $2 - user (default: postgres)
#   $3 - password (optional)
#   Additional args passed to psql
########################
postgresql_execute() {
  local db="${1:-}"
  local user="${2:-postgres}"
  local pass="${3:-}"
  shift 3 || true
  local args=("-U" "$user" "-p" "$POSTGRESQL_PORT_NUMBER" "-h" "127.0.0.1")
  [[ -n "$db" ]] && args+=("-d" "$db")
  PGPASSWORD="$pass" psql "${args[@]}" "$@"
}

########################
# Initialize master database
########################
postgresql_master_init_db() {
  local initdb_args=()
  if [[ -n "$POSTGRESQL_INITDB_ARGS" ]]; then
    read -r -a initdb_args <<< "$POSTGRESQL_INITDB_ARGS"
  fi
  if [[ -n "$POSTGRESQL_INITDB_WAL_DIR" ]]; then
    ensure_dir_exists "$POSTGRESQL_INITDB_WAL_DIR"
    initdb_args+=("--waldir" "$POSTGRESQL_INITDB_WAL_DIR")
  fi
  info "Initializing PostgreSQL database..."
  initdb -E UTF8 -D "$POSTGRESQL_DATA_DIR" -U "postgres" "${initdb_args[@]}"
}

########################
# Initialize slave database from master
########################
postgresql_slave_init_db() {
  info "Waiting for replication master at $POSTGRESQL_MASTER_HOST:$POSTGRESQL_MASTER_PORT_NUMBER..."
  local ready_counter=$POSTGRESQL_INIT_MAX_TIMEOUT
  while ! PGPASSWORD="$POSTGRESQL_REPLICATION_PASSWORD" pg_isready -U "$POSTGRESQL_REPLICATION_USER" -h "$POSTGRESQL_MASTER_HOST" -p "$POSTGRESQL_MASTER_PORT_NUMBER" -d "postgres"; do
    sleep 1
    ready_counter=$((ready_counter - 1))
    if [[ $ready_counter -le 0 ]]; then
      error "PostgreSQL master is not ready after $POSTGRESQL_INIT_MAX_TIMEOUT seconds"
      exit 1
    fi
  done
  info "Replicating initial database..."
  PGPASSWORD="$POSTGRESQL_REPLICATION_PASSWORD" pg_basebackup -D "$POSTGRESQL_DATA_DIR" -U "$POSTGRESQL_REPLICATION_USER" -h "$POSTGRESQL_MASTER_HOST" -p "$POSTGRESQL_MASTER_PORT_NUMBER" -X stream -w -v -P
}

########################
# Start PostgreSQL in background
########################
postgresql_start_bg() {
  info "Starting PostgreSQL in background..."
  pg_ctl start -W -D "$POSTGRESQL_DATA_DIR" -l "$POSTGRESQL_LOG_FILE" -o "--config-file=$POSTGRESQL_CONF_FILE --external_pid_file=$POSTGRESQL_PID_FILE --hba_file=$POSTGRESQL_PGHBA_FILE"
  local counter=$POSTGRESQL_INIT_MAX_TIMEOUT
  while ! pg_isready -U "postgres" -p "$POSTGRESQL_PORT_NUMBER" -h "127.0.0.1" >/dev/null 2>&1; do
    sleep 1
    counter=$((counter - 1))
    if [[ $counter -le 0 ]]; then
      error "PostgreSQL is not ready after $POSTGRESQL_INIT_MAX_TIMEOUT seconds"
      exit 1
    fi
  done
}

########################
# Stop PostgreSQL
########################
postgresql_stop() {
  if [[ -f "$POSTGRESQL_PID_FILE" ]]; then
    info "Stopping PostgreSQL..."
    pg_ctl stop -w -D "$POSTGRESQL_DATA_DIR" -m "$POSTGRESQL_SHUTDOWN_MODE" -t "$POSTGRESQL_PGCTLTIMEOUT"
  fi
}

########################
# Alter postgres user password
########################
postgresql_alter_postgres_user() {
  local escaped_password="${1//\'/\'\'}"
  info "Changing password of postgres..."
  echo "ALTER ROLE postgres WITH PASSWORD '$escaped_password';" | postgresql_execute
}

########################
# Create admin user
########################
postgresql_create_admin_user() {
  local escaped_password="${POSTGRESQL_PASSWORD//\'/\'\'}"
  local postgres_password="${POSTGRESQL_POSTGRES_PASSWORD:-$POSTGRESQL_PASSWORD}"
  info "Creating user $POSTGRESQL_USERNAME..."
  echo "CREATE ROLE \"$POSTGRESQL_USERNAME\" WITH LOGIN CREATEDB PASSWORD '$escaped_password';" | postgresql_execute "" "postgres" "$postgres_password"
  info "Granting privileges on database $POSTGRESQL_DATABASE to $POSTGRESQL_USERNAME..."
  echo "GRANT ALL PRIVILEGES ON DATABASE \"$POSTGRESQL_DATABASE\" TO \"$POSTGRESQL_USERNAME\";" | postgresql_execute "" "postgres" "$postgres_password"
  echo "ALTER DATABASE \"$POSTGRESQL_DATABASE\" OWNER TO \"$POSTGRESQL_USERNAME\";" | postgresql_execute "" "postgres" "$postgres_password"
  echo "ALTER SCHEMA public OWNER TO \"$POSTGRESQL_USERNAME\";" | postgresql_execute "$POSTGRESQL_DATABASE" "postgres" "$postgres_password"
}

########################
# Create custom database
########################
postgresql_create_custom_database() {
  local db_name="${1:?missing database}"
  echo "CREATE DATABASE \"$db_name\"" | postgresql_execute "" "postgres" ""
}

########################
# Create replication user
########################
postgresql_create_replication_user() {
  local escaped_password="${POSTGRESQL_REPLICATION_PASSWORD//\'/\'\'}"
  local postgres_password="${POSTGRESQL_POSTGRES_PASSWORD:-$POSTGRESQL_PASSWORD}"
  info "Creating replication user $POSTGRESQL_REPLICATION_USER..."
  echo "CREATE ROLE \"$POSTGRESQL_REPLICATION_USER\" REPLICATION LOGIN ENCRYPTED PASSWORD '$escaped_password'" | postgresql_execute "" "postgres" "$postgres_password"
}

########################
# Execute custom pre-init scripts
########################
postgresql_custom_pre_init_scripts() {
  info "Loading custom pre-init scripts..."
  if [[ -d "$POSTGRESQL_PREINITSCRIPTS_DIR" ]]; then
    find "$POSTGRESQL_PREINITSCRIPTS_DIR/" -type f -name "*.sh" 2>/dev/null | sort | while read -r f; do
      if [[ -x "$f" ]]; then
        debug "Executing $f"
        "$f"
      else
        debug "Sourcing $f"
        . "$f"
      fi
    done
  fi
}

########################
# Execute custom init scripts
########################
postgresql_custom_init_scripts() {
  info "Loading custom init scripts..."
  if [[ -d "$POSTGRESQL_INITSCRIPTS_DIR" ]] && [[ ! -f "$POSTGRESQL_VOLUME_DIR/.user_scripts_initialized" ]]; then
    postgresql_start_bg
    find "$POSTGRESQL_INITSCRIPTS_DIR/" -type f -regex ".*\.\(sh\|sql\|sql.gz\)" 2>/dev/null | sort | while read -r f; do
      case "$f" in
        *.sh)
          if [[ -x "$f" ]]; then
            debug "Executing $f"
            "$f"
          else
            debug "Sourcing $f"
            . "$f"
          fi
          ;;
        *.sql)
          debug "Executing $f"
          postgresql_execute "$POSTGRESQL_DATABASE" "$POSTGRESQL_USERNAME" "$POSTGRESQL_PASSWORD" < "$f"
          ;;
        *.sql.gz)
          debug "Executing $f"
          gunzip -c "$f" | postgresql_execute "$POSTGRESQL_DATABASE" "$POSTGRESQL_USERNAME" "$POSTGRESQL_PASSWORD"
          ;;
      esac
    done
    touch "$POSTGRESQL_VOLUME_DIR/.user_scripts_initialized"
  fi
}

########################
# Clean stale files from restart
########################
postgresql_clean_from_restart() {
  local files=(
    "$POSTGRESQL_DATA_DIR/postmaster.pid"
    "$POSTGRESQL_DATA_DIR/standby.signal"
    "$POSTGRESQL_DATA_DIR/recovery.signal"
  )
  for file in "${files[@]}"; do
    if [[ -f "$file" ]]; then
      info "Cleaning stale $file file"
      rm "$file"
    fi
  done
}

########################
# Main initialization
########################
postgresql_initialize() {
  info "Initializing PostgreSQL..."

  rm -f "$POSTGRESQL_PID_FILE"

  # Copy mounted configuration
  if [[ -d "$POSTGRESQL_MOUNTED_CONF_DIR" ]] && compgen -G "$POSTGRESQL_MOUNTED_CONF_DIR"/* >/dev/null 2>&1; then
    debug "Copying files from $POSTGRESQL_MOUNTED_CONF_DIR to $POSTGRESQL_CONF_DIR"
    cp -fr "$POSTGRESQL_MOUNTED_CONF_DIR"/. "$POSTGRESQL_CONF_DIR"
  fi

  # Create directories
  for dir in "$POSTGRESQL_TMP_DIR" "$POSTGRESQL_LOG_DIR" "$POSTGRESQL_DATA_DIR" "$POSTGRESQL_CONF_DIR" "$POSTGRESQL_CONF_DIR/conf.d"; do
    ensure_dir_exists "$dir"
  done
  chmod u+rwx "$POSTGRESQL_DATA_DIR"
  chmod go-rwx "$POSTGRESQL_DATA_DIR"

  # Check if fresh install or existing data
  if [[ ! -d "$POSTGRESQL_DATA_DIR/base" ]]; then
    if [[ "$POSTGRESQL_REPLICATION_MODE" = "master" ]] || [[ -z "$POSTGRESQL_REPLICATION_MODE" ]]; then
      postgresql_master_init_db
      postgresql_create_config
      postgresql_create_pghba
      postgresql_start_bg
      [[ -n "$POSTGRESQL_DATABASE" ]] && [[ "$POSTGRESQL_DATABASE" != "postgres" ]] && postgresql_create_custom_database "$POSTGRESQL_DATABASE"
      if [[ "$POSTGRESQL_USERNAME" = "postgres" ]]; then
        postgresql_alter_postgres_user "$POSTGRESQL_PASSWORD"
      else
        [[ -n "$POSTGRESQL_POSTGRES_PASSWORD" ]] && postgresql_alter_postgres_user "$POSTGRESQL_POSTGRES_PASSWORD"
        postgresql_create_admin_user
      fi
      postgresql_restrict_pghba
      [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && postgresql_create_replication_user
      [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && postgresql_configure_replication_parameters
      [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && postgresql_add_replication_to_pghba
      [[ ${POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS:-0} -gt 0 ]] && postgresql_configure_synchronous_replication
      postgresql_configure_fsync
      is_boolean_yes "$POSTGRESQL_ENABLE_TLS" && postgresql_configure_tls
    else
      postgresql_slave_init_db
      postgresql_create_config
      postgresql_create_pghba
      postgresql_restrict_pghba
      postgresql_configure_replication_parameters
      postgresql_configure_fsync
      is_boolean_yes "$POSTGRESQL_ENABLE_TLS" && postgresql_configure_tls
      postgresql_configure_recovery
    fi
  else
    info "Deploying PostgreSQL with persisted data..."
    export POSTGRESQL_FIRST_BOOT="no"
    postgresql_create_config
    postgresql_create_pghba
    postgresql_restrict_pghba
    postgresql_configure_replication_parameters
    postgresql_configure_fsync
    is_boolean_yes "$POSTGRESQL_ENABLE_TLS" && postgresql_configure_tls
    [[ "$POSTGRESQL_REPLICATION_MODE" = "master" ]] && [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && postgresql_add_replication_to_pghba
    [[ "$POSTGRESQL_REPLICATION_MODE" = "master" ]] && [[ ${POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS:-0} -gt 0 ]] && postgresql_configure_synchronous_replication
    [[ "$POSTGRESQL_REPLICATION_MODE" = "slave" ]] && postgresql_configure_recovery
  fi

  [[ -n "$POSTGRESQL_SHARED_PRELOAD_LIBRARIES" ]] && postgresql_set_property "shared_preload_libraries" "$POSTGRESQL_SHARED_PRELOAD_LIBRARIES"

  # Delete conf files from data dir
  rm -f "$POSTGRESQL_DATA_DIR/postgresql.conf" "$POSTGRESQL_DATA_DIR/pg_hba.conf"

  postgresql_stop
}

# Run initialization
postgresql_clean_from_restart
postgresql_custom_pre_init_scripts
postgresql_initialize
postgresql_custom_init_scripts
postgresql_stop

if is_boolean_yes "$POSTGRESQL_ALLOW_REMOTE_CONNECTIONS"; then
  info "Remote connections enabled"
fi

info "PostgreSQL initialization complete"
