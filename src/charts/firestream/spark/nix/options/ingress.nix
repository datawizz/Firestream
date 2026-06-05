# Spark chart options: `ingress.*`.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.spark.ingress = mkOption {
    type = types.nullOr t.ingressType;
    default = null;
    description = "Ingress resource configuration for accessing the Spark master UI";
  };
}
