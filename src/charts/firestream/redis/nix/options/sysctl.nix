# Redis chart options: `sysctl.*` (init container to modify kernel settings).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.sysctl = mkOption {
    default = null;
    description = "init-sysctl container parameters (modify kernel settings)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable init container to modify Kernel settings";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "OS Shell + Utility image for sysctl init container";
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default init-sysctl container command";
        };

        mountHostSys = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount the host `/sys` folder to `/host-sys`";
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
      };
    });
  };
}
