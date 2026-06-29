# Persistent Volume Claim (persistence) type definition
#
# Model A: every leaf is nullOr-wrapped and defaults to null so unset fields
# are stripped from the generated values.yaml and Helm falls back to the
# chart's bundled defaults. A fully-unset persistence block therefore
# serialises to `{}` and is a harmless no-op merge over the chart's own
# bundled values.yaml.
#
# This is the canonical Bitnami `persistence.*` shape, derived as a superset
# of the hand-rolled persistence blocks across the postgresql / redis / kafka
# / odoo / airflow / jupyterhub chart overlays. Free-form map fields
# (annotations / labels / selector / dataSource) use `types.attrs` to stay
# faithful to arbitrary Bitnami values shapes.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # Bitnami persistence (PVC) configuration type
  #
  # freeformType lets a chart carry PVC leaves outside this shared superset
  # without being rejected, mirroring the hand-rolled overlays which used
  # `freeformType = types.attrsOf types.anything` on their persistence blocks.
  persistenceType = types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable persistence using Persistent Volume Claims";
        example = true;
      };

      storageClass = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "PVC Storage Class for the volume";
        example = "standard";
      };

      accessModes = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Persistent Volume access modes";
        example = [ "ReadWriteOnce" ];
      };

      accessMode = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Persistent Volume access mode (DEPRECATED: use accessModes instead)";
        example = "ReadWriteOnce";
      };

      size = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "PVC Storage Request size for the volume";
        example = "8Gi";
      };

      existingClaim = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Name of an existing PVC to use for persistence";
        example = "my-existing-pvc";
      };

      volumeName = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Name to assign the volume";
        example = "data";
      };

      mountPath = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Path where the volume will be mounted";
        example = "/bitnami/data";
      };

      subPath = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Subdirectory of the volume to mount at the mountPath";
      };

      resourcePolicy = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Set to \"keep\" to avoid removing the PVC during a helm delete operation";
        example = "keep";
      };

      annotations = mkOption {
        type = types.nullOr types.attrs;
        default = null;
        description = "Annotations for the PVC";
        example = { "helm.sh/resource-policy" = "keep"; };
      };

      labels = mkOption {
        type = types.nullOr types.attrs;
        default = null;
        description = "Labels for the PVC";
      };

      selector = mkOption {
        type = types.nullOr types.attrs;
        default = null;
        description = "Selector to match an existing Persistent Volume";
        example = { matchLabels = { app = "my-app"; }; };
      };

      dataSource = mkOption {
        type = types.nullOr types.attrs;
        default = null;
        description = "Custom PVC data source";
      };
    };
  };
}
