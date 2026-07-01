# Odoo chart options: `volumePermissions.*` (init container that chowns
# the PV mount point).
#
# Optional init container that runs before the main odoo container to
# change ownership of the persistent volume mount point to
# `runAsUser:fsGroup`. Disabled by default; enable when the PV's
# StorageClass requires it.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.odoo.volumePermissions = mkOption {
    default = null;
    description = "Init container that changes the owner/group of the PV mount point";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable init container that chowns the PV mount point";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset for the init container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements for the init container";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Init container's container security context";
        };
      };
    });
  };
}
