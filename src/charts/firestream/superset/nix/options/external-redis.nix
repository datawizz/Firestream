# Superset chart options: `externalRedis.*`.
#
# Active only when `redis.enabled = false`. The superset celery configuration
# reads these to point at the external Redis broker.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset.externalRedis = mkOption {
    default = null;
    description = "External Redis connection (used when redis.enabled = false)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        host = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis host";
        };

        port = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Redis port number";
        };

        username = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis username (most Redis deployments don't require one)";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis password";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret containing the Redis credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Key inside the existing secret holding the Redis password";
        };
      };
    });
  };
}
