# Airflow chart options: `externalRedis.*` (use an external Redis).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.externalRedis = mkOption {
    default = null;
    description = "External Redis configuration (used when redis.enabled = false)";
    type = types.nullOr (types.submodule {
      options = {
        host = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis host";
        };

        port = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Redis port number";
        };

        username = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis username (retrieves the password from the secret if empty)";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis password";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret resource containing the Redis credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret key containing the Redis password";
        };
      };
    });
  };
}
