# Airflow chart options: `ingress.*` (Airflow web Ingress).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.ingress = mkOption {
    type = types.nullOr t.ingressType;
    default = null;
    description = "Airflow web ingress parameters (enabled/ingressClassName/pathType/apiVersion/hostname/path/annotations/tls/selfSigned/extra*/secrets/extraRules)";
  };
}
