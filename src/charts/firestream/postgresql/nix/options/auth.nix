# PostgreSQL chart options: `auth.*` (database credentials).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.auth = mkOption {
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

        postgresPassword = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the 'postgres' admin user";
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

        replicationUsername = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the replication user";
        };

        replicationPassword = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password for the replication user";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing secret to use for PostgreSQL credentials";
        };

        secretKeys = mkOption {
          default = null;
          description = "Names of keys in the existing secret to use for PostgreSQL credentials";
          type = types.nullOr (types.submodule {
            options = {
              adminPasswordKey = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of key for admin password";
              };

              userPasswordKey = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of key for user password";
              };

              replicationPasswordKey = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of key for replication password";
              };
            };
          });
        };

        usePasswordFiles = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount credentials as files instead of using environment variables";
        };
      };
    });
  };
}
