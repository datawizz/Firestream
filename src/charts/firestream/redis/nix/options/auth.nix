# Redis chart options: `auth.*` (Redis credentials + ACL).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis.auth = mkOption {
    default = null;
    description = "Redis authentication parameters";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable password authentication";
        };

        sentinel = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable authentication on sentinels too";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Redis password (defaults to a random 10-character alphanumeric string if not set)";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret with Redis credentials";
        };

        existingSecretPasswordKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password key to be retrieved from existing secret (ignored unless `auth.existingSecret` is set)";
        };

        usePasswordFiles = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount credentials as files instead of using environment variables";
        };

        usePasswordFileFromSecret = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount password file from secret";
        };

        acl = mkOption {
          default = null;
          description = "Redis ACL configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enables the support of the Redis ACL system";
              };

              sentinel = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enables the support of the Redis ACL system for Sentinel Nodes";
              };

              users = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "A list of the configured users in the Redis ACL system";
              };

              userSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the Secret containing user credentials for ACL users";
              };
            };
          });
        };
      };
    });
  };
}
