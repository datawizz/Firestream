# Kafka chart options: `rbac.*` (Role + RoleBinding for the SA so pods can
# query the K8s API e.g. for auto-discovery).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.rbac = mkOption {
    default = null;
    description = "RBAC configuration";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether to create RBAC resources binding the SA to a role that allows querying the K8s API";
        };
      };
    });
  };
}
