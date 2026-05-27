# Superset (v5) Container Options
# Copyright Firestream. MIT License.
#
# Externalized, declarative configuration for the Superset v5 container, consumed
# by bin/nix/firestream/containers/eval-container.nix. Defaults here are lifted
# VERBATIM from module.nix so that evalContainer's default build is byte-for-byte
# identical to the legacy flake.nix build path.
#
# IMPORTANT: env defaults use a PER-LEAF mkDefault (each value wrapped
# individually). A single mkDefault around the whole attrset would be replaced
# wholesale when a consumer overrides one key, silently dropping siblings.

{ lib, pkgs, ... }:

{
  config.superset = {
    version = lib.mkDefault "5";

    python = lib.mkDefault pkgs.python311;

    # Paths configuration
    # Per-key mkDefault so individual paths can be overridden independently.
    paths = {
      base = lib.mkDefault "/opt/superset";
      conf = lib.mkDefault "/opt/superset";
      data = lib.mkDefault "/opt/superset/superset_home";
      logs = lib.mkDefault "/opt/superset/logs";
    };

    # Environment variables with defaults (preserving Bitnami compatibility)
    # CRITICAL: per-leaf mkDefault (wrap each value), NOT a whole-set mkDefault.
    env = builtins.mapAttrs (_: lib.mkDefault) {
      # Paths
      SUPERSET_HOME = "/opt/superset/superset_home";
      SUPERSET_CONFIG_PATH = "/opt/superset/superset_config.py";
      SUPERSET_LOG_DIR = "/opt/superset/logs";
      SUPERSET_TMP_DIR = "/opt/superset/tmp";
      FLASK_APP = "superset.app:create_app()";
      PYTHONPATH = "/app/pythonpath";

      # User configuration
      SUPERSET_USERNAME = "admin";
      SUPERSET_PASSWORD = "admin";
      SUPERSET_FIRSTNAME = "Superset";
      SUPERSET_LASTNAME = "Admin";
      SUPERSET_EMAIL = "admin@superset.local";

      # Webserver configuration
      SUPERSET_WEBSERVER_HOST = "0.0.0.0";
      SUPERSET_WEBSERVER_PORT_NUMBER = "8088";
      SUPERSET_WEBSERVER_WORKERS = "4";
      SUPERSET_WEBSERVER_WORKER_CLASS = "gthread";
      SUPERSET_WEBSERVER_THREADS = "20";
      SUPERSET_WEBSERVER_TIMEOUT = "60";
      SUPERSET_WEBSERVER_KEEPALIVE = "2";
      SUPERSET_WEBSERVER_MAX_REQUESTS = "0";
      SUPERSET_WEBSERVER_MAX_REQUESTS_JITTER = "0";
      SUPERSET_WEBSERVER_LIMIT_REQUEST_LINE = "0";
      SUPERSET_WEBSERVER_LIMIT_REQUEST_FIELD_SIZE = "0";
      SUPERSET_WEBSERVER_ACCESS_LOG_FILE = "-";
      SUPERSET_WEBSERVER_ERROR_LOG_FILE = "-";

      # Celery Flower configuration
      FLOWER_BASIC_AUTH = "";

      # Database configuration (metadata database)
      SUPERSET_DATABASE_DIALECT = "postgresql";
      SUPERSET_DATABASE_HOST = "postgresql";
      SUPERSET_DATABASE_PORT_NUMBER = "5432";
      SUPERSET_DATABASE_NAME = "superset";
      SUPERSET_DATABASE_USERNAME = "superset";
      SUPERSET_DATABASE_PASSWORD = "";
      SUPERSET_DATABASE_USE_SSL = "no";

      # Redis configuration (cache and Celery broker)
      REDIS_HOST = "redis";
      REDIS_PORT_NUMBER = "6379";
      REDIS_USER = "";
      REDIS_PASSWORD = "";
      REDIS_CELERY_DATABASE = "0";
      REDIS_RESULTS_DATABASE = "1";
      REDIS_CACHE_DATABASE = "2";
      REDIS_USE_SSL = "no";

      # Role configuration (webserver, celery-worker, celery-beat, celery-flower, init)
      SUPERSET_ROLE = "webserver";

      # Init configuration
      SUPERSET_LOAD_EXAMPLES = "true";  # Default to loading demo data
      SUPERSET_SKIP_DATABASE_WAIT = "no";
      SUPERSET_IMPORT_DATASOURCES = "";

      # Examples database (defaults to metadata DB if empty)
      # Used by SUPERSET_LOAD_EXAMPLES to store demo data
      EXAMPLES_DATABASE_DIALECT = "";  # Empty = use SUPERSET_DATABASE_DIALECT
      EXAMPLES_DATABASE_HOST = "";     # Empty = use SUPERSET_DATABASE_HOST
      EXAMPLES_DATABASE_PORT_NUMBER = "";
      EXAMPLES_DATABASE_NAME = "";     # Empty = use SUPERSET_DATABASE_NAME
      EXAMPLES_DATABASE_USER = "";
      EXAMPLES_DATABASE_PASSWORD = "";
      EXAMPLES_DATABASE_USE_SSL = "";

      # Python cache
      PYTHONPYCACHEPREFIX = "/opt/superset/tmp/pycache";

      # Debug mode
      BITNAMI_DEBUG = "false";
    };

    # Variables that support Docker secrets (_FILE suffix).
    # Whole-value mkDefault is correct for lists (replacement semantics).
    envSecrets = lib.mkDefault [
      "SUPERSET_SECRET_KEY"
      "SUPERSET_USERNAME"
      "SUPERSET_PASSWORD"
      "SUPERSET_FIRSTNAME"
      "SUPERSET_LASTNAME"
      "SUPERSET_EMAIL"
      "SUPERSET_DATABASE_HOST"
      "SUPERSET_DATABASE_PORT_NUMBER"
      "SUPERSET_DATABASE_NAME"
      "SUPERSET_DATABASE_USERNAME"
      "SUPERSET_DATABASE_PASSWORD"
      "SUPERSET_DATABASE_USE_SSL"
      "REDIS_HOST"
      "REDIS_PORT_NUMBER"
      "REDIS_USER"
      "REDIS_PASSWORD"
      "REDIS_CELERY_DATABASE"
      "REDIS_RESULTS_DATABASE"
      "REDIS_CACHE_DATABASE"
      "REDIS_USE_SSL"
      "FLOWER_BASIC_AUTH"
      "SUPERSET_LOAD_EXAMPLES"
      "SUPERSET_SKIP_DATABASE_WAIT"
      "SUPERSET_IMPORT_DATASOURCES"
      "SUPERSET_ROLE"
      "SUPERSET_WEBSERVER_HOST"
      "SUPERSET_WEBSERVER_PORT_NUMBER"
      "SUPERSET_WEBSERVER_WORKERS"
      "SUPERSET_WEBSERVER_TIMEOUT"
      # Examples database (for demo data)
      "EXAMPLES_DATABASE_DIALECT"
      "EXAMPLES_DATABASE_HOST"
      "EXAMPLES_DATABASE_PORT_NUMBER"
      "EXAMPLES_DATABASE_NAME"
      "EXAMPLES_DATABASE_USER"
      "EXAMPLES_DATABASE_PASSWORD"
      "EXAMPLES_DATABASE_USE_SSL"
    ];

    exposedPorts = lib.mkDefault [ 8088 5555 ];  # 8088=webserver, 5555=flower
  };
}
