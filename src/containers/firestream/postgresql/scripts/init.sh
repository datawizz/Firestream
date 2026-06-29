# PostgreSQL initialization logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: Sub-helper function definitions (postgresql_execute,
# postgresql_master_init_db, postgresql_create_admin_user, etc.) live in
# scripts/helpers.sh and are emitted at top-level of libpostgresql.sh so
# chart init containers can `source /opt/bitnami/scripts/libpostgresql.sh`
# and call them directly.
#
# postgresql_initialize MUST stay nested here — its name collides with the
# engine's <name>_initialize wrapper. Bash function shadowing redefines the
# outer wrapper at runtime, which is the same pattern Bitnami uses.

info "Initializing PostgreSQL..."

########################
# Main initialization (nested inside engine's postgresql_initialize wrapper —
# redefines the wrapper via function shadowing).
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
