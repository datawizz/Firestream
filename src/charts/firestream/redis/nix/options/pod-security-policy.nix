# Redis chart options: `podSecurityPolicy.*` (DEPRECATED in K8s v1.25+).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis.podSecurityPolicy = mkOption {
    default = null;
    description = "PodSecurityPolicy configuration (DEPRECATED — removed in K8s v1.25+)";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether to create a PodSecurityPolicy";
        };

        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable PodSecurityPolicy's RBAC rules";
        };
      };
    });
  };
}
