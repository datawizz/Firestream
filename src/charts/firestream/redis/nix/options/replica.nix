# Redis chart options: `replica.*` (the replicas StatefulSet, used when
# architecture is `replication`).
#
# Same shape as master.nix but with replica-specific extras (replicaCount,
# externalMaster bootstrap, autoscaling). Top-level commonly-touched keys are
# typed; rest accepted via freeformType.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.replica = mkOption {
    default = null;
    description = "Redis replicas configuration parameters";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        kind = mkOption {
          type = types.nullOr (types.enum [ "DaemonSet" "StatefulSet" ]);
          default = null;
          description = "Use either DaemonSet or StatefulSet (default)";
        };

        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of Redis replicas to deploy";
        };

        revisionHistoryLimit = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of old history to retain to allow rollback";
        };

        configuration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Configuration for Redis replicas nodes";
        };

        disableCommands = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Array with Redis commands to disable on replicas nodes";
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default container command";
        };

        args = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default container args";
        };

        enableServiceLinks = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether information about services should be injected into pod's environment variable";
        };

        preExecCmds = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Additional commands to run prior to starting Redis replicas";
        };

        extraFlags = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Array with additional command line flags for Redis replicas";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for Redis replicas nodes";
        };

        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing ConfigMap containing extra env vars";
        };

        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing Secret containing extra env vars";
        };

        externalMaster = mkOption {
          default = null;
          description = "Use external master for bootstrapping replicas";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Use external master for bootstrapping";
              };

              host = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "External master host to bootstrap from";
              };

              port = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Port for Redis service external master host";
              };
            };
          });
        };

        containerPorts = mkOption {
          default = null;
          description = "Container ports for Redis replicas nodes";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              redis = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Container port to open on Redis replicas nodes";
              };
            };
          });
        };

        livenessProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Liveness probe configuration";
        };

        readinessProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Readiness probe configuration";
        };

        startupProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Startup probe configuration";
        };

        customLivenessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom liveness probe";
        };

        customReadinessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom readiness probe";
        };

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Set container resources according to one common preset";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements";
        };

        podSecurityContext = mkOption {
          type = types.nullOr t.podSecurityContext;
          default = null;
          description = "Pod Security Context";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container Security Context";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Alternate scheduler for Redis replicas pods";
        };

        updateStrategy = mkOption {
          type = types.nullOr t.updateStrategyType;
          default = null;
          description = "Update strategy";
        };

        minReadySeconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "How many seconds a pod needs to be ready before killing the next, during update";
        };

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
        };

        podManagementPolicy = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "podManagementPolicy to manage scaling operation";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount Service Account token in pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Redis replicas pods host aliases";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for Redis replicas pods";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for Redis replicas pods";
        };

        shareProcessNamespace = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Share a single process namespace between all containers in replicas pods";
        };

        podAffinityPreset = mkOption {
          type = types.nullOr (types.enum [ "" "soft" "hard" ]);
          default = null;
          description = "Pod affinity preset";
        };

        podAntiAffinityPreset = mkOption {
          type = types.nullOr (types.enum [ "" "soft" "hard" ]);
          default = null;
          description = "Pod anti-affinity preset";
        };

        nodeAffinityPreset = mkOption {
          type = types.nullOr t.nodeAffinityPreset;
          default = null;
          description = "Node affinity preset configuration";
        };

        affinity = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Affinity for Redis replicas pods assignment";
        };

        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node selector";
        };

        tolerations = mkOption {
          type = types.nullOr (types.listOf t.tolerationType);
          default = null;
          description = "Tolerations";
        };

        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf t.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints";
        };

        dnsPolicy = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "DNS Policy for Redis replica pods";
        };

        dnsConfig = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "DNS Configuration for Redis replica pods";
        };

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Lifecycle hooks for the Redis replica container";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume mounts";
        };

        sidecars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional sidecar containers";
        };

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional init containers";
        };

        persistence = mkOption {
          default = null;
          description = "Persistence configuration for Redis replicas nodes";
          type = types.nullOr t.persistenceType;
        };

        persistentVolumeClaimRetentionPolicy = mkOption {
          default = null;
          description = "PVC retention policy for the StatefulSet";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Controls if and how PVCs are deleted during the lifecycle of a StatefulSet";
              };

              whenScaled = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when replica count is reduced";
              };

              whenDeleted = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when StatefulSet is deleted";
              };
            };
          });
        };

        service = mkOption {
          type = types.nullOr t.serviceType;
          default = null;
          description = "Replica service configuration";
        };

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Termination grace period in seconds";
        };

        autoscaling = mkOption {
          default = null;
          description = "Autoscaling configuration for replicas";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable replica autoscaling settings";
              };

              minReplicas = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Minimum replicas for the pod autoscaling";
              };

              maxReplicas = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Maximum replicas for the pod autoscaling";
              };

              targetCPU = mkOption {
                # Bitnami: "" by default, can be int (percentage).
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Percentage of CPU to consider when autoscaling";
              };

              targetMemory = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Percentage of Memory to consider when autoscaling";
              };
            };
          });
        };

        serviceAccount = mkOption {
          default = null;
          description = "Replica ServiceAccount configuration";
          type = types.nullOr (types.submodule {
            options = {
              create = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Specifies whether a ServiceAccount should be created";
              };

              name = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the ServiceAccount to use";
              };

              automountServiceAccountToken = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Auto-mount the service account token";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Annotations for the ServiceAccount";
              };
            };
          });
        };

        pdb = mkOption {
          type = types.nullOr t.pdbType;
          default = null;
          description = "Pod Disruption Budget configuration";
        };

        extraPodSpec = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra PodSpec for the Redis replicas pod(s)";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Additional custom annotations for Redis replicas resource";
        };
      };
    });
  };
}
