# PostgreSQL chart options: `volumePermissions.*` (init-container that fixes
# ownership on the persistent volume).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.volumePermissions = mkOption {
    default = null;
    description = "Init container for fixing volume permissions";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the init container that changes the owner and group of the persistent volume";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Init container volume-permissions image";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset for the init container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Resource requirements for the init container";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Init container security context";
        };
      };
    });
  };
}
