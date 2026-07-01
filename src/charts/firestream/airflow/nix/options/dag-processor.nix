# Airflow chart options: `dagProcessor.*`.
#
# Plain Airflow component (incl. its own `enabled` flag): every values.yaml key
# is covered by the shared component base type (mkComponentType). No extra keys.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.dagProcessor = mkOption {
    default = { };
    description = "Airflow DAG processor parameters";
    type = t.mkComponentType { componentName = "dagProcessor"; };
  };
}
