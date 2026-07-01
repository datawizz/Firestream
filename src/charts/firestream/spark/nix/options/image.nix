# Spark chart options: `image.*` (main Spark image).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.spark.image = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "Spark image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
  };
}
