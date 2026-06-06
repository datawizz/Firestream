# Kafka chart options: `serviceAccount.*` (chart-wide ServiceAccount).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration for Kafka pods";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable creation of ServiceAccount for Kafka pods";
        };

        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the service account to use";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Allows auto-mount of ServiceAccountToken";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Additional custom annotations for the ServiceAccount";
        };
      };
    });
  };
}
