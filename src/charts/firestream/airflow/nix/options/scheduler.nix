# Airflow chart options: `scheduler.*`.
#
# The scheduler is a plain Airflow component: all of its values.yaml keys are
# covered by the shared component base type (mkComponentType). No extra keys.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.scheduler = mkOption {
    default = { };
    description = "Airflow scheduler parameters";
    type = t.mkComponentType { componentName = "scheduler"; };
  };
}
