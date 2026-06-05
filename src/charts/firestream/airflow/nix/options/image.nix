# Airflow chart options: `image.*` (main Airflow image).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.image = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "Airflow image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
  };
}
