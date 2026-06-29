# SeaweedFS chart options: `allInOne.*` (single-pod topology).
#
# Firestream runs SeaweedFS as one pod (`weed server -master -volume -filer
# -s3`). We turn the topology on, enable the embedded S3 gateway with auth, and
# declare a default `firestream` bucket created by the post-install hook.
#
# Storage: the upstream chart defaults `allInOne.data.type` to "emptyDir",
# which is the right single-node dev default (writable /data, no PVC, no
# storageclass dependency). We pin it explicitly for clarity.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.seaweedfs.allInOne = mkOption {
    default = null;
    description = "All-in-one single-pod SeaweedFS topology.";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Run SeaweedFS as a single all-in-one pod.";
        };

        s3 = mkOption {
          default = null;
          description = "Embedded S3 gateway settings for the all-in-one pod.";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable the embedded S3 gateway.";
              };
              enableAuth = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable S3 auth for the all-in-one pod.";
              };
              createBuckets = mkOption {
                default = null;
                description = "Buckets created by the post-install hook.";
                # freeform passthrough: only `name` is modelled; consumers may
                # add anonymousRead/ttl/versioning/objectLock per the chart's
                # createBuckets schema and they flow straight through.
                type = types.nullOr (types.listOf (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options.name = mkOption {
                    type = types.str;
                    description = "Bucket name.";
                  };
                }));
              };
            };
          });
        };

        data = mkOption {
          default = null;
          description = "All-in-one /data volume configuration.";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr (types.enum [ "hostPath" "persistentVolumeClaim" "emptyDir" "existingClaim" ]);
                default = null;
                description = "Volume backing for /data.";
              };
            };
          });
        };
      };
    });
  };
}
