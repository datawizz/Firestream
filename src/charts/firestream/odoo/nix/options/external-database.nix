# Odoo chart options: `externalDatabase.*`.
#
# Active only when `postgresql.enabled = false`. The odoo
# `externaldb-secrets.yaml` template writes a `<release>-externaldb`
# Secret holding `password` (from externalDatabase.password) and
# `postgres-password` (from externalDatabase.postgresqlPostgresPassword)
# only when `externalDatabase.existingSecret` is also empty.
#
# Odoo's externalDatabase carries TWO password fields (the
# application-user password and a separate admin/`postgres` user
# password — the latter is used to run schema migrations / create the
# odoo database when `externalDatabase.create = true`).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.odoo.externalDatabase = mkOption {
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
          description = "Non-root username for Odoo";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the non-root username for Odoo";
        };

        database = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Odoo database name";
        };

        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable PostgreSQL user and database creation (when using an external db)";
        };

        postgresqlPostgresUser = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "External Database admin username";
        };

        postgresqlPostgresPassword = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "External Database admin password";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret resource containing the database credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the non-root credentials";
        };

        existingSecretPostgresPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the admin credentials";
        };
      };
    });
  };
}
