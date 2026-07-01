# Superset chart options: `serviceAccount.*` (top-level ServiceAccount).
#
# A single ServiceAccount shared by all superset pods (web/worker/beat/
# flower/init). Standard Bitnami shape.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration for all Superset pods";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable ServiceAccount creation";
        };

        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the ServiceAccount to use";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Additional ServiceAccount annotations (evaluated as a template)";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Automount the ServiceAccount token";
        };
      };
    });
  };
}
