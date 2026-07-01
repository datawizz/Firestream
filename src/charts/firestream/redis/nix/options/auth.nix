# Redis chart options: `auth.*` (Redis credentials + ACL).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.auth = mkOption {
    default = null;
    description = "Redis authentication parameters";
    type = types.nullOr t.secretType;
  };
}
