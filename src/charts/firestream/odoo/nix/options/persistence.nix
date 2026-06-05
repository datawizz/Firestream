# Odoo chart options: `persistence.*` (PVC for Odoo data).
#
# Standard Bitnami persistence shape: enabled / storageClass /
# accessModes / size / annotations / selector / existingClaim plus
# the deprecated `accessMode` singular.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.odoo.persistence = mkOption {
    default = null;
    description = "Persistence configuration for Odoo data (PVC)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable persistence using Persistent Volume Claims";
        };

        subPath = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Include subPath on volumeMounts for Odoo data";
        };

        resourcePolicy = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Set to \"keep\" to avoid removing PVCs during a helm delete operation";
        };

        storageClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Persistent Volume storage class";
        };

        accessModes = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Persistent Volume access modes";
        };

        accessMode = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Persistent Volume access mode (DEPRECATED: use accessModes instead)";
        };

        size = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Persistent Volume size";
        };

        dataSource = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom PVC data source";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the PVC";
        };

        selector = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Selector to match an existing Persistent Volume";
        };

        existingClaim = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "The name of an existing PVC to use for persistence";
        };
      };
    });
  };
}
