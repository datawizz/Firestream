# Redis chart options: `serviceAccount.*` (top-level chart-wide ServiceAccount).
#
# NOTE: master.serviceAccount and replica.serviceAccount are component-scoped
# and declared in master.nix / replica.nix.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Specifies whether a ServiceAccount should be created";
        };

        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the ServiceAccount to use";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Auto-mount the service account token";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the ServiceAccount";
        };
      };
    });
  };
}
