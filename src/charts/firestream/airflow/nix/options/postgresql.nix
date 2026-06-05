# Airflow chart options: `postgresql.*` (bundled Bitnami PostgreSQL subchart).
#
# Subchart passthrough: the most-commonly-toggled keys (enabled / auth.* /
# architecture) are first-class typed options; every other PostgreSQL subchart
# value (primary.*, readReplicas.*, tls.*, metrics.*, etc.) is accepted via the
# submodule's free-form type so the whole subchart surface can be overridden
# without re-declaring it here. Model A: all first-class leaves nullOr/default
# = null so unset keys are stripped from the generated values.yaml.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.postgresql = mkOption {
    default = null;
    description = "Bundled PostgreSQL subchart parameters (passthrough)";
    type = types.nullOr (types.submodule {
      # Free-form: accept any additional PostgreSQL subchart key verbatim.
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Switch to enable or disable the PostgreSQL helm chart";
        };

        architecture = mkOption {
          type = types.nullOr (types.enum [ "standalone" "replication" ]);
          default = null;
          description = "PostgreSQL architecture (standalone or replication)";
        };

        auth = mkOption {
          default = null;
          description = "PostgreSQL authentication parameters";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enablePostgresUser = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Assign a password to the 'postgres' admin user";
              };

              username = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name for a custom user to create";
              };

              password = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Password for the custom user to create";
              };

              database = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name for a custom database to create";
              };

              existingSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of existing secret to use for PostgreSQL credentials";
              };
            };
          });
        };
      };
    });
  };
}
