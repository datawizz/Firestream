# PostgreSQL chart options: `serviceAccount.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.serviceAccount = mkOption {
    default = null;
    description = "ServiceAccount configuration";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable creation of ServiceAccount for PostgreSQL pod";
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
