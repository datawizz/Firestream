# JupyterHub chart options: `externalDatabase.*`.
#
# Active only when `postgresql.enabled = false`. The hub.configuration
# heredoc reads these to build the db.url connection string for jupyterhub.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.jupyterhub.externalDatabase = mkOption {
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
          description = "Non-root username for JupyterHub";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the non-root username for JupyterHub";
        };

        database = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "JupyterHub database name";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret resource containing the database credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the database credentials";
        };
      };
    });
  };
}
