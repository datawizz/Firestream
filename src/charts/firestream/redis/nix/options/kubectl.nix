# Redis chart options: `kubectl.*` (init container used by Sentinel to update
# the isMaster label on Redis pods).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.kubectl = mkOption {
    default = null;
    description = "Kubectl init container parameters";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Kubectl init container image";
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Kubectl command to execute";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container Security Context for the kubectl init container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Resource requirements for the kubectl init container";
        };
      };
    });
  };
}
