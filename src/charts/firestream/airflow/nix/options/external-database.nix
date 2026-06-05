# Airflow chart options: `externalDatabase.*` (use an external PostgreSQL).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.externalDatabase = mkOption {
    default = null;
    description = "External PostgreSQL configuration (used when postgresql.enabled = false)";
    type = types.nullOr (types.submodule {
      options = {
        host = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Database host";
        };

        port = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Database port number";
        };

        user = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Non-root username for Airflow";
        };

        database = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Airflow database name";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the non-root username for Airflow";
        };

        sqlConnection = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "SQL connection string (overrides host/port/user/database/password if set)";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret resource containing the database credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the database password";
        };

        existingSecretSqlConnectionKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the SQL connection string";
        };
      };
    });
  };
}
