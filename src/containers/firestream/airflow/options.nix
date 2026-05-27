# Airflow Container Options
# Copyright Firestream. MIT License.
#
# Externalized, declarative configuration for the Airflow container, consumed
# by bin/nix/firestream/containers/eval-container.nix. Defaults here are lifted
# VERBATIM from module.nix so that evalContainer's default build is byte-for-byte
# identical to the legacy flake.nix build path.
#
# IMPORTANT: env defaults use a PER-LEAF mkDefault (each value wrapped
# individually). A single mkDefault around the whole attrset would be replaced
# wholesale when a consumer overrides one key, silently dropping siblings.
# Per-leaf mkDefault lets a consumer override a single env var while every other
# default is preserved.

{ lib, pkgs, ... }:

{
  config.airflow = {
    version = lib.mkDefault "3.0.3";

    python = lib.mkDefault pkgs.python312;

    # Paths configuration (Airflow uses /opt/airflow structure)
    # Per-key mkDefault so individual paths can be overridden independently.
    paths = {
      base = lib.mkDefault "/opt/airflow";
      conf = lib.mkDefault "/opt/airflow";
      data = lib.mkDefault "/opt/airflow/dags";
      logs = lib.mkDefault "/opt/airflow/logs";
    };

    # Environment variables with defaults (from env-defaults.sh)
    # CRITICAL: per-leaf mkDefault (wrap each value), NOT a whole-set mkDefault.
    env = builtins.mapAttrs (_: lib.mkDefault) {
      # Paths
      AIRFLOW_HOME = "/opt/airflow";
      AIRFLOW_DAGS_DIR = "/opt/airflow/dags";
      AIRFLOW_LOGS_DIR = "/opt/airflow/logs";
      AIRFLOW_SCHEDULER_LOGS_DIR = "/opt/airflow/logs/scheduler";
      AIRFLOW_PLUGINS_DIR = "/opt/airflow/plugins";
      AIRFLOW_TMP_DIR = "/opt/airflow/tmp";
      AIRFLOW_CONF_FILE = "/opt/airflow/airflow.cfg";
      AIRFLOW_WEBSERVER_CONF_FILE = "/opt/airflow/webserver_config.py";

      # User configuration
      AIRFLOW_USERNAME = "admin";
      AIRFLOW_PASSWORD = "admin";
      AIRFLOW_FIRSTNAME = "Firstname";
      AIRFLOW_LASTNAME = "Lastname";
      AIRFLOW_EMAIL = "user@example.com";

      # Component configuration
      AIRFLOW_COMPONENT_TYPE = "api-server";
      AIRFLOW_EXECUTOR = "LocalExecutor";
      AIRFLOW_SKIP_DB_SETUP = "no";
      AIRFLOW_LOAD_EXAMPLES = "yes";
      AIRFLOW_STANDALONE_DAG_PROCESSOR = "no";
      AIRFLOW_DB_MIGRATE_TIMEOUT = "120";
      AIRFLOW_FORCE_OVERWRITE_CONF_FILE = "no";

      # API Server / Webserver configuration
      AIRFLOW_APISERVER_HOST = "127.0.0.1";
      AIRFLOW_APISERVER_PORT_NUMBER = "8080";
      AIRFLOW_WEBSERVER_SECRET_KEY = "airflow-web-server-key";
      AIRFLOW_APISERVER_SECRET_KEY = "airflow-api-server-key";
      AIRFLOW_JWT_SECRET_KEY = "";  # Auto-generated if empty
      AIRFLOW_ENABLE_HTTPS = "no";
      AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER = "80";

      # Triggerer configuration
      AIRFLOW_TRIGGERER_DEFAULT_CAPACITY = "1000";

      # Database configuration
      AIRFLOW_DATABASE_HOST = "postgresql";
      AIRFLOW_DATABASE_PORT_NUMBER = "5432";
      AIRFLOW_DATABASE_NAME = "airflow";
      AIRFLOW_DATABASE_USERNAME = "airflow";
      AIRFLOW_DATABASE_USE_SSL = "no";
      AIRFLOW_DATABASE_PASSWORD = "";

      # Redis (Celery) configuration
      REDIS_HOST = "redis";
      REDIS_PORT_NUMBER = "6379";
      REDIS_DATABASE = "1";
      REDIS_PASSWORD = "";
      AIRFLOW_REDIS_USE_SSL = "no";

      # LDAP configuration
      AIRFLOW_LDAP_ENABLE = "no";
      AIRFLOW_LDAP_USER_REGISTRATION = "True";
      AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN = "True";
      AIRFLOW_LDAP_USE_TLS = "False";
      AIRFLOW_LDAP_ALLOW_SELF_SIGNED = "True";

      # Python cache
      PYTHONPYCACHEPREFIX = "/opt/airflow/tmp/pycache";

      # Debug mode
      BITNAMI_DEBUG = "false";
    };

    # Variables that support Docker secrets (_FILE suffix).
    # Whole-value mkDefault is correct for lists (replacement semantics).
    envSecrets = lib.mkDefault [
      "AIRFLOW_USERNAME"
      "AIRFLOW_PASSWORD"
      "AIRFLOW_FIRSTNAME"
      "AIRFLOW_LASTNAME"
      "AIRFLOW_EMAIL"
      "AIRFLOW_COMPONENT_TYPE"
      "AIRFLOW_EXECUTOR"
      "AIRFLOW_RAW_FERNET_KEY"
      "AIRFLOW_FORCE_OVERWRITE_CONF_FILE"
      "AIRFLOW_FERNET_KEY"
      "AIRFLOW_WEBSERVER_SECRET_KEY"
      "AIRFLOW_APISERVER_SECRET_KEY"
      "AIRFLOW_JWT_SECRET_KEY"
      "AIRFLOW_APISERVER_BASE_URL"
      "AIRFLOW_APISERVER_HOST"
      "AIRFLOW_APISERVER_PORT_NUMBER"
      "AIRFLOW_LOAD_EXAMPLES"
      "AIRFLOW_HOSTNAME_CALLABLE"
      "AIRFLOW_POOL_NAME"
      "AIRFLOW_POOL_SIZE"
      "AIRFLOW_POOL_DESC"
      "AIRFLOW_STANDALONE_DAG_PROCESSOR"
      "AIRFLOW_TRIGGERER_DEFAULT_CAPACITY"
      "AIRFLOW_WORKER_QUEUE"
      "AIRFLOW_SKIP_DB_SETUP"
      "AIRFLOW_DB_MIGRATE_TIMEOUT"
      "AIRFLOW_ENABLE_HTTPS"
      "AIRFLOW_EXTERNAL_APISERVER_PORT_NUMBER"
      "AIRFLOW_DATABASE_HOST"
      "AIRFLOW_DATABASE_PORT_NUMBER"
      "AIRFLOW_DATABASE_NAME"
      "AIRFLOW_DATABASE_USERNAME"
      "AIRFLOW_DATABASE_PASSWORD"
      "AIRFLOW_DATABASE_USE_SSL"
      "AIRFLOW_REDIS_USE_SSL"
      "REDIS_HOST"
      "REDIS_PORT_NUMBER"
      "REDIS_USER"
      "REDIS_PASSWORD"
      "REDIS_DATABASE"
      "AIRFLOW_LDAP_ENABLE"
      "AIRFLOW_LDAP_URI"
      "AIRFLOW_LDAP_SEARCH"
      "AIRFLOW_LDAP_UID_FIELD"
      "AIRFLOW_LDAP_BIND_USER"
      "AIRFLOW_LDAP_BIND_PASSWORD"
      "AIRFLOW_LDAP_USER_REGISTRATION"
      "AIRFLOW_LDAP_USER_REGISTRATION_ROLE"
      "AIRFLOW_LDAP_ROLES_MAPPING"
      "AIRFLOW_LDAP_ROLES_SYNC_AT_LOGIN"
      "AIRFLOW_LDAP_USE_TLS"
      "AIRFLOW_LDAP_ALLOW_SELF_SIGNED"
      "AIRFLOW_LDAP_TLS_CA_CERTIFICATE"
    ];

    exposedPorts = lib.mkDefault [ 8080 8125 8793 8794 ];

    # image: schema defaults (repository = "firestream-airflow", tag = null -> version)
    # are already correct; do not set here.
  };
}
