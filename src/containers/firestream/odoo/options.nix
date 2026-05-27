# Odoo Container Options (shared base)
# Copyright Firestream. MIT License.
#
# Externalized, declarative SHARED configuration for the Odoo container, consumed
# by bin/nix/firestream/containers/eval-container.nix. Defaults here are lifted
# VERBATIM from module.nix so that evalContainer's default build is byte-for-byte
# identical to the legacy flake.nix build path.
#
# NOTE: Odoo is MULTI-VERSION (15/16/17/18). `version` and `python` vary per
# build and are supplied by the flake-module via an inline override module, NOT
# here. This base module holds only the shared env/paths/ports/secrets.
#
# IMPORTANT: env defaults use a PER-LEAF mkDefault (each value wrapped
# individually). A single mkDefault around the whole attrset would be replaced
# wholesale when a consumer overrides one key, silently dropping siblings.

{ lib, config, ... }:

{
  config.odoo = {
    # Paths configuration
    # Per-key mkDefault so individual paths can be overridden independently.
    # KEEP "/opt/odoo/log" exactly (do NOT normalise to "logs").
    paths = {
      base = lib.mkDefault "/opt/odoo";
      conf = lib.mkDefault "/opt/odoo/conf";
      data = lib.mkDefault "/opt/odoo/data";
      logs = lib.mkDefault "/opt/odoo/log";
    };

    # Environment variables with defaults
    # CRITICAL: per-leaf mkDefault (wrap each value), NOT a whole-set mkDefault.
    # ODOO_VERSION tracks config.odoo.version (supplied per-build by the
    # flake-module's override module), matching the legacy `ODOO_VERSION = odooVersion`.
    env = builtins.mapAttrs (_: lib.mkDefault) {
      # Paths
      ODOO_BASE_DIR = "/opt/odoo";
      ODOO_BIN_DIR = "/opt/odoo/bin";
      ODOO_CONF_DIR = "/opt/odoo/conf";
      ODOO_CONF_FILE = "/opt/odoo/conf/odoo.conf";
      ODOO_DATA_DIR = "/opt/odoo/data";
      ODOO_ADDONS_DIR = "/opt/odoo/addons";
      ODOO_TMP_DIR = "/opt/odoo/tmp";
      ODOO_PID_FILE = "/opt/odoo/tmp/odoo.pid";
      ODOO_LOGS_DIR = "/opt/odoo/log";
      ODOO_LOG_FILE = "/opt/odoo/log/odoo-server.log";

      # Volume paths
      ODOO_VOLUME_DIR = "/opt/odoo";

      # User/group
      ODOO_DAEMON_USER = "odoo";
      ODOO_DAEMON_GROUP = "odoo";

      # Port configuration
      ODOO_PORT_NUMBER = "8069";
      ODOO_LONGPOLLING_PORT_NUMBER = "8072";

      # Bootstrap configuration
      ODOO_SKIP_BOOTSTRAP = "no";
      ODOO_SKIP_MODULES_UPDATE = "no";
      ODOO_LOAD_DEMO_DATA = "no";
      ODOO_LIST_DB = "no";

      # Odoo credentials
      ODOO_EMAIL = "admin";
      ODOO_PASSWORD = "admin";

      # SMTP configuration
      ODOO_SMTP_HOST = "";
      ODOO_SMTP_PORT_NUMBER = "";
      ODOO_SMTP_USER = "";
      ODOO_SMTP_PASSWORD = "";
      ODOO_SMTP_PROTOCOL = "";

      # Database configuration
      ODOO_DATABASE_HOST = "postgresql";
      ODOO_DATABASE_PORT_NUMBER = "5432";
      ODOO_DATABASE_NAME = "firestream_odoo";
      ODOO_DATABASE_USER = "firestream";
      ODOO_DATABASE_PASSWORD = "";
      ODOO_DATABASE_FILTER = "";

      # Timeouts
      ODOO_DB_WAIT_TIMEOUT = "120";

      # Empty password flag
      ALLOW_EMPTY_PASSWORD = "no";

      # Debug mode
      BITNAMI_DEBUG = "false";

      # Odoo version (for scripts) - tracks config.odoo.version
      ODOO_VERSION = config.odoo.version;
    };

    # Variables that support Docker secrets (_FILE suffix).
    # Whole-value mkDefault is correct for lists (replacement semantics).
    envSecrets = lib.mkDefault [
      "ODOO_PASSWORD"
      "ODOO_DATABASE_PASSWORD"
      "ODOO_SMTP_PASSWORD"
      "ODOO_SMTP_HOST"
      "ODOO_SMTP_PORT_NUMBER"
      "ODOO_SMTP_USER"
      "ODOO_SMTP_PROTOCOL"
      "ODOO_DATABASE_HOST"
      "ODOO_DATABASE_PORT_NUMBER"
      "ODOO_DATABASE_NAME"
      "ODOO_DATABASE_USER"
      "ODOO_DATABASE_FILTER"
      "ODOO_EMAIL"
      "ODOO_SKIP_BOOTSTRAP"
      "ODOO_SKIP_MODULES_UPDATE"
      "ODOO_LOAD_DEMO_DATA"
      "ODOO_LIST_DB"
    ];

    exposedPorts = lib.mkDefault [ 8069 8072 ];
  };
}
