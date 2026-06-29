# Superset chart options: `auth.*` (web UI admin user).
#
# secret.yaml writes:
#   superset-password    <- providedValues = list "auth.password",     randomised when empty
#   superset-secret-key  <- providedValues = list "auth.secretKey",    randomised when empty (length 42)
# `auth.existingSecret` skips the in-chart secret entirely; the keys
# `superset-password` and `superset-secret-key` are then read from the
# external secret.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.superset.auth = mkOption {
    default = null;
    description = "Superset admin web-UI authentication configuration";
    type = types.nullOr t.secretType;
  };
}
