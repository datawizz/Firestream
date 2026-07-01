# Airflow chart options: `redis.*` (bundled Bitnami Redis subchart).
#
# Subchart passthrough (same approach as postgresql.nix): enabled / auth.* /
# architecture are first-class typed options; the rest of the Redis subchart
# surface (master.*, replica.*, sentinel.*, etc.) is accepted via the
# submodule free-form type. Model A: nullOr/default = null.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.redis = mkOption {
    default = null;
    description = "Bundled Redis subchart parameters (passthrough)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Switch to enable or disable the Redis helm chart";
        };

        architecture = mkOption {
          type = types.nullOr (types.enum [ "standalone" "replication" ]);
          default = null;
          description = "Redis architecture (standalone or replication)";
        };

        auth = mkOption {
          default = null;
          description = "Redis authentication parameters";
          type = types.nullOr t.secretType;
        };
      };
    });
  };
}
