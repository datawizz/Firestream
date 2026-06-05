# PostgreSQL chart options: `image.*` (main PostgreSQL image).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.image = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "PostgreSQL image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
  };
}
