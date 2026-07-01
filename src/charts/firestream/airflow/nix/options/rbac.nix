# Airflow chart options: `rbac.*`.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow.rbac = mkOption {
    default = null;
    description = "RBAC configuration";
    type = types.nullOr (types.submodule {
      options = {
        create = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether to create & use RBAC resources or not";
        };

        rules = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Custom RBAC rules to set";
        };
      };
    });
  };
}
