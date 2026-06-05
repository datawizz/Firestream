# Redis chart options: `image.*` (main Redis image).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.image = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "Redis image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
  };
}
