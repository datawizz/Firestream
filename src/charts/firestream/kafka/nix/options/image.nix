# Kafka chart options: `image.*` (main Kafka image).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.kafka.image = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "Kafka image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
  };
}
