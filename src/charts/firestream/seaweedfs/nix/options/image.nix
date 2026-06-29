# SeaweedFS chart options: `image.*` (top-level image block).
#
# The chart's `seaweedfs.image` helper (templates/shared/_helpers.tpl) reads
# `.Values.image.{registry,repository,tag}`. This is the exact slot the
# Firestream image-injector writes into (componentPath = [ "image" ]); the
# Firestream defaults stay null here so injection fills them.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.seaweedfs.image = mkOption {
    default = null;
    description = "Top-level SeaweedFS image (registry/repository/tag). Filled by image injection.";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        registry = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Image registry (e.g. docker.io). Null => no registry prefix.";
        };
        repository = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Image repository (e.g. firestream-seaweedfs).";
        };
        tag = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Image tag.";
        };
      };
    });
  };
}
