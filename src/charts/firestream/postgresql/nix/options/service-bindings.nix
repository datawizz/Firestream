# PostgreSQL chart options: `serviceBindings.*` (Service Binding spec support).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql.serviceBindings = mkOption {
    default = null;
    description = "Service binding configuration (experimental, ref: https://servicebinding.io)";
    type = types.nullOr (types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Create secret for service binding";
        };
      };
    });
  };
}
