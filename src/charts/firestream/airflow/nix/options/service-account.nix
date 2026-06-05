# Airflow chart options: `serviceAccount.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration for the Airflow pods";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable creation of ServiceAccount for Airflow pods";
        };

        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the created ServiceAccount";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Auto-mount the service account token in the pod";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for service account";
        };
      };
    });
  };
}
