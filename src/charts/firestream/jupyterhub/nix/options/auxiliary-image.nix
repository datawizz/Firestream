# JupyterHub chart options: `auxiliaryImage.*`.
#
# Top-level image triple used by helper init containers (os-shell). The
# chart shapes it like a Bitnami image block, so we re-use chartTypes.imageType.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.auxiliaryImage = mkOption {
    type = types.nullOr t.imageType;
    default = null;
    description = "Auxiliary helper image (os-shell), used by init containers";
  };
}
