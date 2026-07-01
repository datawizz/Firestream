# Airflow chart options: `auth.*` (web UI credentials & app secrets).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.auth = mkOption {
    default = null;
    description = "Airflow authentication / secret-key parameters";
    type = types.nullOr t.secretType;
  };
}
