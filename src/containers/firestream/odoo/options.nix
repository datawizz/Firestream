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

let
  # Spec for one vendored addon repository. `src` (a derivation/path), when set,
  # WINS over owner/repo/rev/hash so non-GitHub sources work too. See
  # ./vendor-addons.nix for how these are turned into a baked
  # /opt/firestream/odoo/vendor-addons/<module> tree.
  vendoredAddonSpec = lib.types.submodule {
    options = {
      name = lib.mkOption {
        type = lib.types.str;
        description = "Label for this repo (used in build logs and collision errors).";
      };
      owner = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "GitHub owner (when fetching via fetchFromGitHub).";
      };
      repo = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "GitHub repo (when fetching via fetchFromGitHub).";
      };
      rev = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Commit or tag to pin (when fetching via fetchFromGitHub).";
      };
      hash = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "fetchFromGitHub sha256 (SRI string).";
      };
      src = lib.mkOption {
        type = lib.types.nullOr (lib.types.either lib.types.path lib.types.package);
        default = null;
        description = ''
          A prebuilt source tree (derivation or path). When set, it WINS over
          owner/repo/rev/hash, enabling non-GitHub sources (fetchgit, local path,
          flake input).
        '';
      };
      sourceRoot = lib.mkOption {
        type = lib.types.str;
        default = ".";
        description = "Subdirectory inside the repo that contains the module dirs.";
      };
      modules = lib.mkOption {
        type = lib.types.nullOr (lib.types.listOf lib.types.str);
        default = null;
        description = ''
          Explicit subset of module directory names to vendor. When null
          (default), every immediate child dir with an Odoo manifest
          (__manifest__.py / __openerp__.py) is auto-discovered.
        '';
      };
    };
  };
in
{
  # Build-time addon vendoring: a list of addon-repo specs baked into the image at
  # /opt/firestream/odoo/vendor-addons and appended to addons_path (see module.nix). Empty by
  # default — stock images are unchanged. Forwarded to the container factory via
  # `extraModuleArgs.vendoredAddons` below (the eval-container.nix seam).
  options.odoo.vendoredAddons = lib.mkOption {
    type = lib.types.listOf vendoredAddonSpec;
    default = [ ];
    description = ''
      Third-party Odoo addon repositories to vendor at build time. Each entry is
      fetched (GitHub by default, or any `src`), and its modules are laid into a
      baked read-only /opt/firestream/odoo/vendor-addons directory wired into addons_path.
    '';
  };

  config.odoo = {
    # Forward the vendored-addons list to module.nix through the factory's
    # extraModuleArgs seam (eval-container.nix splices this into moduleArgs).
    extraModuleArgs.vendoredAddons = config.odoo.vendoredAddons;

    # Paths configuration
    # Per-key mkDefault so individual paths can be overridden independently.
    # KEEP "/opt/firestream/odoo/log" exactly (do NOT normalise to "logs").
    paths = {
      base = lib.mkDefault "/opt/firestream/odoo";
      conf = lib.mkDefault "/opt/firestream/odoo/conf";
      data = lib.mkDefault "/firestream/odoo/data";
      logs = lib.mkDefault "/opt/firestream/odoo/log";
    };

    # Environment variables with defaults
    # CRITICAL: per-leaf mkDefault (wrap each value), NOT a whole-set mkDefault.
    # ODOO_VERSION tracks config.odoo.version (supplied per-build by the
    # flake-module's override module), matching the legacy `ODOO_VERSION = odooVersion`.
    env = builtins.mapAttrs (_: lib.mkDefault) {
      # Paths
      ODOO_BASE_DIR = "/opt/firestream/odoo";
      ODOO_BIN_DIR = "/opt/firestream/odoo/bin";
      ODOO_CONF_DIR = "/opt/firestream/odoo/conf";
      ODOO_CONF_FILE = "/opt/firestream/odoo/conf/odoo.conf";
      ODOO_DATA_DIR = "/firestream/odoo/data";
      ODOO_ADDONS_DIR = "/opt/firestream/odoo/addons";
      ODOO_TMP_DIR = "/opt/firestream/odoo/tmp";
      ODOO_PID_FILE = "/opt/firestream/odoo/tmp/odoo.pid";
      ODOO_LOGS_DIR = "/opt/firestream/odoo/log";
      ODOO_LOG_FILE = "/opt/firestream/odoo/log/odoo-server.log";

      # Volume paths
      ODOO_VOLUME_DIR = "/firestream/odoo";

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

      # Empty password flag — default to yes for out-of-the-box local/dev/e2e
      # use; production overrides via the standard mkDefault seam.
      ALLOW_EMPTY_PASSWORD = "yes";

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

    # Phase 4: enable in-image firestream-healthd. Odoo's /web/database/selector
    # is publicly available once the http worker is up; we use curl --fail to
    # treat any non-2xx as not-yet-ready. `ODOO_PORT_NUMBER` defaults to 8069.
    health = {
      enable = lib.mkDefault true;
      readinessCmd = lib.mkDefault
        ''curl -fsS -o /dev/null "http://localhost:''${ODOO_PORT_NUMBER:-8069}/web/database/selector"'';
    };

    # Odoo requires a postgres metadata DB; embed it as a dependency sub-service
    # so `.#odoo-up` is a working out-of-the-box stack. Mirrors the airflow /
    # superset pattern. Whole-block mkDefault: a consumer override supplies its
    # own complete topology.
    compose = lib.mkDefault {
      projectName = "firestream-odoo";
      dependencies = [ "postgresql" ];

      # +34000 host-port offset. Each of the 8 canonical apps gets a DISTINCT
      # offset (spacing 2000) so all 8 can run simultaneously on docker without
      # host-port collisions. odoo=34000.
      #   postgresql 5432 -> host 39432
      #   odoo       8069 -> host 42069
      #   odoo gevent 8072 -> host 42072
      #   healthd    9180 -> host 43180
      hostPortOffset = 34000;

      sharedEnv = {
        ODOO_DATABASE_HOST = "postgresql";
        ODOO_DATABASE_NAME = "firestream_odoo";
        ODOO_DATABASE_USER = "odoo";
        ODOO_DATABASE_PASSWORD = "odoo";
      };

      volumes = {
        postgresql_data = { };
      };

      services = {
        postgresql = {
          image = "@postgresql";
          env = {
            POSTGRESQL_DATABASE = "firestream_odoo";
            POSTGRESQL_USERNAME = "odoo";
            POSTGRESQL_PASSWORD = "odoo";
            ALLOW_EMPTY_PASSWORD = "no";
          };
          ports = [ "5432:5432" ];
          volumes = [ "postgresql_data:/firestream/postgresql" ];
          healthcheck = {
            test = [
              "CMD"
              "bash"
              "-c"
              "exec 3<>/dev/tcp/127.0.0.1/9180 && printf 'GET /readyz HTTP/1.0\\r\\n\\r\\n' >&3 && head -n 1 <&3 | grep -q ' 200'"
            ];
            interval = "10s";
            timeout = "5s";
            retries = 5;
            start_period = "30s";
          };
        };
        odoo = {
          # Own firestream-odoo image; publish web + longpolling + healthd.
          ports = [ "8069:8069" "8072:8072" "9180:9180" ];
          dependsOn = [ "postgresql" ];
        };
      };
    };
  };
}
