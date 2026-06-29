# Nextjs chart options: `image.*`.
#
# Single Bitnami-shaped image triple (registry/repository/tag/digest/
# pullPolicy/pullSecrets) plus a `debug` flag. This is the only image
# shipped by the Bitnami nextjs chart (no separate worker/init images).
#
# We use chartTypes.imageType (a submodule with
# `freeformType = types.attrsOf types.anything`) so nextjs-specific extras
# like `debug` flow through unchanged.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.nextjs.image = mkOption {
    default = null;
    description = "Nextjs image (registry/repository/tag/digest/pullPolicy/pullSecrets/debug)";
    type = types.nullOr t.imageType;
  };
}
