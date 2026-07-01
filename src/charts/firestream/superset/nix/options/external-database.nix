# Superset chart options: `externalDatabase.*`.
#
# Active only when `postgresql.enabled = false`. The superset configmap /
# secret templates read these to build the SQLAlchemy connection URI.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset.externalDatabase = mkOption {
    default = null;
    description = "External PostgreSQL connection (used when postgresql.enabled = false)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        host = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Database host";
        };

        port = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Database port number";
        };

        user = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Non-root username for Superset";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the non-root username for Superset";
        };

        database = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Superset database name";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret containing the database credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Key inside the existing secret holding the database password";
        };
      };
    });
  };
}
