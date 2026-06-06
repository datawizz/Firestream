# Kafka chart options: `global.*`
#
# Model A: every leaf is `nullOr`-wrapped and defaults to null so the
# generated values.yaml stays a sparse override file. Mirrors values.yaml
# `global:` 1:1. The `freeformType` keeps unknown global keys
# (cross-subchart pass-throughs) round-trippable.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.global = mkOption {
    default = null;
    description = "Global Docker image parameters and cross-subchart settings";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        imageRegistry = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Global Docker image registry";
        };

        imagePullSecrets = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Global Docker registry secret names as an array";
        };

        defaultStorageClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Global default StorageClass for Persistent Volume(s)";
        };

        storageClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "DEPRECATED: use global.defaultStorageClass instead";
        };

        security = mkOption {
          default = null;
          description = "Global security settings";
          type = types.nullOr (types.submodule {
            options = {
              allowInsecureImages = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allows skipping image verification";
              };
            };
          });
        };

        compatibility = mkOption {
          default = null;
          description = "Compatibility adaptations for Kubernetes platforms";
          type = types.nullOr (types.submodule {
            options = {
              openshift = mkOption {
                default = null;
                description = "Adaptations for OpenShift";
                type = types.nullOr (types.submodule {
                  options = {
                    adaptSecurityContext = mkOption {
                      type = types.nullOr (types.enum [ "auto" "force" "disabled" ]);
                      default = null;
                      description = "Adapt the securityContext sections of the deployment to make them work on OpenShift";
                    };
                  };
                });
              };
            };
          });
        };
      };
    });
  };
}
