# Airflow chart options: `postgresql.*` (bundled Bitnami PostgreSQL subchart).
#
# Subchart passthrough: the most-commonly-toggled keys (enabled / auth.* /
# architecture) are first-class typed options; every other PostgreSQL subchart
# value (primary.*, readReplicas.*, tls.*, metrics.*, etc.) is accepted via the
# submodule's free-form type so the whole subchart surface can be overridden
# without re-declaring it here. Model A: all first-class leaves nullOr/default
# = null so unset keys are stripped from the generated values.yaml.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.postgresql = mkOption {
    default = null;
    description = "Bundled PostgreSQL subchart parameters (passthrough)";
    type = types.nullOr (types.submodule {
      # Free-form: accept any additional PostgreSQL subchart key verbatim.
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Switch to enable or disable the PostgreSQL helm chart";
        };

        architecture = mkOption {
          type = types.nullOr (types.enum [ "standalone" "replication" ]);
          default = null;
          description = "PostgreSQL architecture (standalone or replication)";
        };

        auth = mkOption {
          default = null;
          description = "PostgreSQL authentication parameters";
          type = types.nullOr t.secretType;
        };
      };
    });
  };
}
