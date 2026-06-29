# Secret / credential (secretType) type definition
#
# Model A: every leaf is nullOr-wrapped and defaults to null so unset fields
# are stripped from the generated values.yaml and Helm falls back to the
# chart's bundled defaults. A fully-unset secret block therefore serialises
# to `{}` and is a harmless no-op merge over the chart's own bundled
# values.yaml.
#
# This is the canonical Bitnami credential/`auth.*` shape, derived as a
# superset of the hand-rolled password / existingSecret blocks across the
# postgresql / redis / airflow / superset / odoo / jupyterhub chart overlays.
# The free-form `secretKeys` map (postgresql uses {adminPasswordKey,
# userPasswordKey, replicationPasswordKey}; other charts use different key
# names) uses `types.attrs` to stay faithful to arbitrary Bitnami shapes.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # Bitnami secret / credential configuration type
  #
  # freeformType lets a chart carry credential leaves outside this shared core
  # (e.g. redis's `enabled` / `sentinel` / `acl`, or chart-specific key names)
  # without being rejected, mirroring the hand-rolled overlays' freeform shape.
  secretType = types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      existingSecret = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Name of an existing secret to use for credentials";
        example = "my-existing-secret";
      };

      secretKeys = mkOption {
        type = types.nullOr types.attrs;
        default = null;
        description = "Names of keys in the existing secret to use for credentials";
        example = {
          adminPasswordKey = "postgres-password";
          userPasswordKey = "password";
        };
      };

      password = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Password (defaults to a random value when unset)";
      };

      passwordKey = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Password key to be retrieved from the existing secret (ignored unless existingSecret is set)";
      };

      existingSecretPasswordKey = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Name of the key in existingSecret holding the password (Bitnami redis/airflow spelling of passwordKey)";
      };

      usePasswordFiles = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Mount credentials as files instead of using environment variables";
      };
    };
  };
}
