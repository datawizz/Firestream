# Container image type definition
#
# Model A: every leaf is nullOr-wrapped and defaults to null so unset fields
# are stripped from the generated values.yaml and Helm falls back to the
# chart's bundled defaults.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # Container image configuration type
  imageType = types.submodule {
    options = {
      registry = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Container image registry";
        example = "gcr.io";
      };

      repository = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Container image repository";
        example = "bitnami/airflow";
      };

      tag = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Container image tag";
        example = "3.0.3-debian-12-r0";
      };

      digest = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Container image digest (overrides tag if set)";
        example = "sha256:abc123...";
      };

      pullPolicy = mkOption {
        type = types.nullOr (types.enum [ "Always" "IfNotPresent" "Never" ]);
        default = null;
        description = "Image pull policy";
      };

      pullSecrets = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Image pull secrets";
        example = [ "my-registry-secret" ];
      };

      debug = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable debug mode in the image";
      };
    };
  };
}
