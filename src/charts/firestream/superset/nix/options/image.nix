# Superset chart options: `image.*`.
#
# Single Bitnami-shaped image triple (registry/repository/tag/digest/
# pullPolicy/pullSecrets) plus a `debug` flag. This image is the one
# shared by the web / worker / beat / flower / init pods.
#
# We extend chartTypes.imageType with `debug` (superset-specific) via a
# freeformType passthrough submodule. The base imageType already declares
# the standard registry/repository/tag/digest/pullPolicy/pullSecrets
# fields with the right Bitnami shapes.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.superset.image = mkOption {
    default = null;
    description = "Superset image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
    # Use the canonical imageType from chartTypes. The base type is a
    # submodule with `freeformType = types.attrsOf types.anything`, so
    # superset-specific extras like `debug` flow through as-is. This
    # keeps the type contract identical to jupyterhub.auxiliaryImage etc.
    type = types.nullOr t.imageType;
  };
}
