# SeaweedFS chart options: `global.*`
#
# Minimal model. Everything passes through from the upstream chart's
# values.yaml `global:` block via freeform passthrough. We only surface
# `global.seaweedfs.loggingLevel` so the Firestream overlay can pin it; the
# image-name fallback (`global.seaweedfs.image.name`) is left untouched because
# image injection writes the top-level `image.*` slot instead.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.seaweedfs.global = mkOption {
    default = null;
    description = "Global Docker image parameters and cross-component SeaweedFS settings.";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        seaweedfs = mkOption {
          default = null;
          description = "SeaweedFS-namespaced global settings.";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              loggingLevel = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "SeaweedFS logging verbosity level (-v flag).";
              };
            };
          });
        };
      };
    });
  };
}
