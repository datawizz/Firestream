# PostgreSQL helper functions
# Copyright Firestream. MIT License.
# These functions are defined at top-level of libpostgresql.sh so chart init
# containers can `source /opt/bitnami/scripts/libpostgresql.sh` and invoke them
# directly. Moved verbatim from init.sh — function bodies unchanged.

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
# Execute SQL statements against a REMOTE PostgreSQL instance.
# Mirrors the Bitnami libpostgresql.sh `postgresql_remote_execute` signature so
# chart init containers (e.g. superset/jupyterhub `wait-for-db`) that run the
# firestream-postgresql image can `source libpostgresql.sh` and call it directly:
#   echo "SELECT 1" | postgresql_remote_execute HOST PORT DB USER PASSWORD
# SQL is read from stdin. Returns psql's exit code (used by retry_while loops).
# Arguments:
#   $1 - hostname (required)
#   $2 - port (required)
#   $3 - database (optional)
#   $4 - user (default: postgres)
#   $5 - password (optional)
#   Additional args passed to psql
########################
postgresql_remote_execute() {
  local -r hostname="${1:?hostname is required}"
  local -r port="${2:?port is required}"
  local db="${3:-}"
  local user="${4:-postgres}"
  local pass="${5:-}"
  shift 5 || true
  local args=("-U" "$user" "-p" "$port" "-h" "$hostname" "-w")
  [[ -n "$db" ]] && args+=("-d" "$db")
  PGPASSWORD="$pass" psql "${args[@]}" "$@"
}

########################
# Execute SQL against a remote PostgreSQL instance and PRINT the query output.
# Bitnami libpostgresql.sh distinguishes a quiet variant (postgresql_remote_execute)
# from a printing one (postgresql_remote_execute_print_output); the superset
# `wait-for-examples` init container greps the printed rows. Same signature as
# postgresql_remote_execute; psql's default output is printed to stdout.
# Arguments: hostname port [database] [user] [password] [extra psql args]
########################
postgresql_remote_execute_print_output() {
  postgresql_remote_execute "$@"
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

# NOTE: postgresql_initialize is NOT included here — it would collide with the
# engine's <name>_initialize wrapper. Its definition lives nested inside
# scripts/init.sh (re-defined at runtime via function shadowing, the same
# pattern Bitnami's libpostgresql.sh uses).
