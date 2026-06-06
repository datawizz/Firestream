# Spark chart options: `serviceAccount.*` (chart-wide ServiceAccount).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.spark.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration for Spark pods";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable creation of ServiceAccount for Spark pods";
        };

        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the service account to use";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the Spark Service Account";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Allow auto-mount of ServiceAccountToken";
        };
      };
    });
  };
}
