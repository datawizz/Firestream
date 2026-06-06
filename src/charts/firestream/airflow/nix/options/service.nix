# Airflow chart options: `service.*` (Airflow web Service).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.service = mkOption {
    type = types.nullOr t.serviceType;
    default = null;
    description = "Airflow web service parameters (type/ports/nodePorts/clusterIP/loadBalancer*/externalTrafficPolicy/sessionAffinity*/annotations/extraPorts)";
  };
}
