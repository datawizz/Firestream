# PostgreSQL Container Options (shared base)
# Copyright Firestream. MIT License.
#
# Externalized, declarative SHARED configuration for the PostgreSQL container,
# consumed by bin/nix/firestream/containers/eval-container.nix. Defaults here are
# lifted VERBATIM from module.nix so that evalContainer's default build is
# byte-for-byte identical to the legacy flake.nix build path.
#
# NOTE: PostgreSQL is MULTI-VERSION (16/17). `version` varies per build and is
# supplied by the flake-module via an inline override module, NOT here.
#
# IMPORTANT: env defaults use a PER-LEAF mkDefault (each value wrapped
# individually). A single mkDefault around the whole attrset would be replaced
# wholesale when a consumer overrides one key, silently dropping siblings.

{ lib, pkgs, ... }:

{
  config.postgresql = {
    # Paths configuration (Bitnami compatibility)
    # Per-key mkDefault so individual paths can be overridden independently.
    paths = {
      base = lib.mkDefault "/opt/firestream/postgresql";
      conf = lib.mkDefault "/opt/firestream/postgresql/conf";
      data = lib.mkDefault "/firestream/postgresql/data";
      logs = lib.mkDefault "/opt/firestream/postgresql/logs";
    };

    # Environment variables with defaults
    # CRITICAL: per-leaf mkDefault (wrap each value), NOT a whole-set mkDefault.
    env = builtins.mapAttrs (_: lib.mkDefault) {
      # Base directories (pre-expanded to avoid alphabetical ordering issues)
      POSTGRESQL_BASE_DIR = "/opt/firestream/postgresql";
      POSTGRESQL_VOLUME_DIR = "/firestream/postgresql";
      POSTGRESQL_DATA_DIR = "/firestream/postgresql/data";
      POSTGRESQL_CONF_DIR = "/opt/firestream/postgresql/conf";
      POSTGRESQL_MOUNTED_CONF_DIR = "/firestream/postgresql/conf";
      POSTGRESQL_DEFAULT_CONF_DIR = "/opt/firestream/postgresql/conf.default";
      POSTGRESQL_CONF_FILE = "/opt/firestream/postgresql/conf/postgresql.conf";
      POSTGRESQL_PGHBA_FILE = "/opt/firestream/postgresql/conf/pg_hba.conf";
      POSTGRESQL_LOG_DIR = "/opt/firestream/postgresql/logs";
      POSTGRESQL_LOG_FILE = "/opt/firestream/postgresql/logs/postgresql.log";
      POSTGRESQL_TMP_DIR = "/opt/firestream/postgresql/tmp";
      POSTGRESQL_PID_FILE = "/opt/firestream/postgresql/tmp/postgresql.pid";
      POSTGRESQL_INITSCRIPTS_DIR = "/docker-entrypoint-initdb.d";
      POSTGRESQL_PREINITSCRIPTS_DIR = "/docker-entrypoint-preinitdb.d";

      # User and group
      POSTGRESQL_DAEMON_USER = "postgres";
      POSTGRESQL_DAEMON_GROUP = "postgres";

      # Connection settings
      POSTGRESQL_PORT_NUMBER = "5432";
      POSTGRESQL_ALLOW_REMOTE_CONNECTIONS = "yes";
      POSTGRESQL_INIT_MAX_TIMEOUT = "60";

      # Authentication
      # Default to ALLOW_EMPTY_PASSWORD=yes so the out-of-the-box image runs
      # in any local/dev/e2e environment without requiring a password override.
      # Production deployers override these via the standard mkDefault seam.
      POSTGRESQL_PASSWORD = "";
      POSTGRESQL_USERNAME = "postgres";
      POSTGRESQL_DATABASE = "";
      POSTGRESQL_POSTGRES_PASSWORD = "";
      ALLOW_EMPTY_PASSWORD = "yes";

      # Replication
      POSTGRESQL_REPLICATION_MODE = "";
      POSTGRESQL_REPLICATION_USER = "";
      POSTGRESQL_REPLICATION_PASSWORD = "";
      POSTGRESQL_MASTER_HOST = "";
      POSTGRESQL_MASTER_PORT_NUMBER = "5432";
      POSTGRESQL_NUM_SYNCHRONOUS_REPLICAS = "0";
      POSTGRESQL_SYNCHRONOUS_COMMIT_MODE = "on";
      POSTGRESQL_CLUSTER_APP_NAME = "walreceiver";
      POSTGRESQL_WAL_LEVEL = "replica";
      POSTGRESQL_REPLICATION_USE_PASSFILE = "no";
      POSTGRESQL_REPLICATION_PASSFILE_PATH = "/opt/firestream/postgresql/conf/.pgpass";

      # TLS
      POSTGRESQL_ENABLE_TLS = "no";
      POSTGRESQL_TLS_CERT_FILE = "";
      POSTGRESQL_TLS_KEY_FILE = "";
      POSTGRESQL_TLS_CA_FILE = "";
      POSTGRESQL_TLS_CRL_FILE = "";
      POSTGRESQL_TLS_PREFER_SERVER_CIPHERS = "yes";

      # LDAP
      POSTGRESQL_ENABLE_LDAP = "no";
      POSTGRESQL_LDAP_URL = "";
      POSTGRESQL_LDAP_SERVER = "";
      POSTGRESQL_LDAP_PORT = "";
      POSTGRESQL_LDAP_PREFIX = "";
      POSTGRESQL_LDAP_SUFFIX = "";
      POSTGRESQL_LDAP_BASE_DN = "";
      POSTGRESQL_LDAP_BIND_DN = "";
      POSTGRESQL_LDAP_BIND_PASSWORD = "";
      POSTGRESQL_LDAP_SEARCH_ATTR = "";
      POSTGRESQL_LDAP_SEARCH_FILTER = "";
      POSTGRESQL_LDAP_TLS = "";
      POSTGRESQL_LDAP_SCHEME = "";

      # Other settings
      POSTGRESQL_SHUTDOWN_MODE = "fast";
      POSTGRESQL_PGCTLTIMEOUT = "60";
      POSTGRESQL_FIRST_BOOT = "yes";
      POSTGRESQL_INITDB_ARGS = "";
      POSTGRESQL_INITDB_WAL_DIR = "";
      POSTGRESQL_SHARED_PRELOAD_LIBRARIES = "";
      POSTGRESQL_FSYNC = "on";
      POSTGRESQL_DEFAULT_TOAST_COMPRESSION = "";
      POSTGRESQL_PASSWORD_ENCRYPTION = "";

      # NSS wrapper
      NSS_WRAPPER_LIB = "${pkgs.nss_wrapper}/lib/libnss_wrapper.so";

      # Debug mode
      BITNAMI_DEBUG = "false";
    };

    # Variables supporting _FILE suffix for Docker secrets.
    # Whole-value mkDefault is correct for lists (replacement semantics).
    envSecrets = lib.mkDefault [
      "POSTGRESQL_PASSWORD"
      "POSTGRESQL_POSTGRES_PASSWORD"
      "POSTGRESQL_REPLICATION_PASSWORD"
      "POSTGRESQL_TLS_CERT_FILE"
      "POSTGRESQL_TLS_KEY_FILE"
      "POSTGRESQL_TLS_CA_FILE"
      "POSTGRESQL_TLS_CRL_FILE"
      "POSTGRESQL_LDAP_URL"
      "POSTGRESQL_LDAP_BIND_PASSWORD"
      "POSTGRESQL_INITDB_ARGS"
      "POSTGRESQL_SHARED_PRELOAD_LIBRARIES"
    ];

    exposedPorts = lib.mkDefault [ 5432 ];

    # Phase 3 pilot / Phase 4 generalisation: enable the in-image
    # firestream-healthd service for postgresql. The entrypoint wrapper
    # (base.nix) launches healthd in the background; /readyz invokes
    # `pg_isready` to track real serviceability, not just listener-bound.
    #
    # The readinessCmd is passed verbatim to `firestream-healthd
    # --readiness-cmd ...`, which evaluates it via `sh -c` at probe time —
    # so `${POSTGRESQL_USERNAME:-postgres}` expands at runtime against the
    # container's own env. This is required for the airflow-embedded
    # postgres, which overrides POSTGRESQL_USERNAME=airflow and
    # POSTGRESQL_DATABASE=airflow; a hard-coded `-U postgres -d postgres`
    # would fail on that deployment. The `:-postgres` defaults keep the
    # standalone case working even if those vars are unset.
    health = {
      enable = lib.mkDefault true;
      # `pg_isready` defaults to `/run/postgresql` for its socket directory.
      # The firestream-postgresql container sockets at POSTGRESQL_TMP_DIR
      # (default `/opt/firestream/postgresql/tmp`), and chart consumers
      # override POSTGRESQL_TMP_DIR to a writable emptyDir mount. In neither
      # case does the default socket directory exist. Pin TCP host+port for
      # a deterministic, cross-deployment probe (matches the Bitnami
      # chart's own readinessProbe pattern: `pg_isready -h 127.0.0.1 -p 5432`).
      readinessCmd = lib.mkDefault
        ''pg_isready -U "''${POSTGRESQL_USERNAME:-postgres}" -d "''${POSTGRESQL_DATABASE:-postgres}" -h 127.0.0.1 -p "''${POSTGRESQL_PORT_NUMBER:-5432}"'';
    };
  };
}
