# PostgreSQL chart options: `auth.*` (database credentials).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.auth = mkOption {
    default = null;
    description = "PostgreSQL authentication parameters";
    type = types.nullOr t.secretType;
  };
}
