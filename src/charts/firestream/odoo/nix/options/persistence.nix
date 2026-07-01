# Odoo chart options: `persistence.*` (PVC for Odoo data).
#
# Standard Bitnami persistence shape: enabled / storageClass /
# accessModes / size / annotations / selector / existingClaim plus
# the deprecated `accessMode` singular.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.odoo.persistence = mkOption {
    default = null;
    description = "Persistence configuration for Odoo data (PVC)";
    type = types.nullOr t.persistenceType;
  };
}
