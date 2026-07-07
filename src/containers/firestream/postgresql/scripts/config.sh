# PostgreSQL configuration generation logic
# Copyright Firestream. MIT License.
# This file is sourced, not executed directly
#
# NOTE: All configuration functions (postgresql_create_config, postgresql_set_property, etc.)
# are defined in postgresqlHelpers in module.nix to ensure they're available in both
# initFn and configFn phases.

info "Generating PostgreSQL configuration..."

# The conf directory ($POSTGRESQL_CONF_DIR, e.g. /opt/firestream/postgresql/conf)
# lives in the container's ephemeral image layer, NOT on the persisted data
# volume. The conf files (postgresql.conf, pg_hba.conf) are written only by the
# init phase (init.sh), which the engine runs solely on first initialization
# (gated by the app-initialized marker on the data volume). When the container is
# recreated against an already-initialized volume — which docker/compose does on
# every image rebuild — the init phase is skipped, so the fresh ephemeral layer
# has no postgresql.conf and Postgres fails to start:
#   postgres: could not access the server configuration file
#   ".../conf/postgresql.conf": No such file or directory
#
# configFn runs on every boot, so regenerate the conf here when it is missing.
# These are pure file writes (mirroring init.sh's persisted-data branch) and need
# no running server, so they are safe to run outside first-time initialization.
if [[ ! -f "$POSTGRESQL_CONF_FILE" ]]; then
  info "postgresql.conf missing (container recreated on an existing data volume) — regenerating configuration..."

  ensure_dir_exists "$POSTGRESQL_CONF_DIR"
  ensure_dir_exists "$POSTGRESQL_CONF_DIR/conf.d"

  # Honour any user-mounted configuration overrides, mirroring init.sh.
  if [[ -d "$POSTGRESQL_MOUNTED_CONF_DIR" ]] && compgen -G "$POSTGRESQL_MOUNTED_CONF_DIR"/* >/dev/null 2>&1; then
    debug "Copying files from $POSTGRESQL_MOUNTED_CONF_DIR to $POSTGRESQL_CONF_DIR"
    cp -fr "$POSTGRESQL_MOUNTED_CONF_DIR"/. "$POSTGRESQL_CONF_DIR"
  fi

  postgresql_create_config
  postgresql_create_pghba
  postgresql_restrict_pghba
  postgresql_configure_replication_parameters
  postgresql_configure_fsync
  is_boolean_yes "$POSTGRESQL_ENABLE_TLS" && postgresql_configure_tls
  [[ "$POSTGRESQL_REPLICATION_MODE" = "master" ]] && [[ -n "$POSTGRESQL_REPLICATION_USER" ]] && postgresql_add_replication_to_pghba
  [[ "$POSTGRESQL_REPLICATION_MODE" = "slave" ]] && postgresql_configure_recovery
  [[ -n "$POSTGRESQL_SHARED_PRELOAD_LIBRARIES" ]] && postgresql_set_property "shared_preload_libraries" "$POSTGRESQL_SHARED_PRELOAD_LIBRARIES"
fi

info "Configuration generation complete"
