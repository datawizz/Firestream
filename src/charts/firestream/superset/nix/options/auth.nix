# Superset chart options: `auth.*` (web UI admin user).
#
# secret.yaml writes:
#   superset-password    <- providedValues = list "auth.password",     randomised when empty
#   superset-secret-key  <- providedValues = list "auth.secretKey",    randomised when empty (length 42)
# `auth.existingSecret` skips the in-chart secret entirely; the keys
# `superset-password` and `superset-secret-key` are then read from the
# external secret.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset.auth = mkOption {
    default = null;
    description = "Superset admin web-UI authentication configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        username = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Username to access the Superset web UI";
        };

        email = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Admin user email address";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Password to access the Superset web UI (random if empty)";
        };

        secretKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Flask SECRET_KEY used to sign session cookies (random 42-char if empty)";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing secret containing `superset-password` and `superset-secret-key`";
        };
      };
    });
  };
}
