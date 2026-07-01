# Kafka chart options: `broker.*` (broker-only StatefulSet).
#
# Broker-only nodes run when controller.controllerOnly=true; they share the
# same pod-spec shape as `controller.*` minus the KRaft-controller-specific
# fields (no controllerOnly, no quorumBootstrapServers). Same shape/strategy
# as controller.nix - typed-and-shallow at the top level, freeform underneath.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.kafka.broker = mkOption {
    default = null;
    description = "Kafka broker-only StatefulSet configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of Kafka broker-only nodes";
        };

        minId = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Minimal node.id values for broker-only nodes";
        };

        config = mkOption {
          type = types.nullOr (types.either types.str (types.attrsOf types.anything));
          default = null;
          description = "Kafka configuration for broker-only nodes";
        };

        overrideConfiguration = mkOption {
          type = types.nullOr (types.either types.str (types.attrsOf types.anything));
          default = null;
          description = "Kafka configuration override (takes precedence over broker.config)";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with the broker Kafka configuration";
        };

        secretConfig = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Additional configuration appended at the end of the generated broker configuration";
        };

        existingSecretConfig = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Secret with additional broker configuration";
        };

        heapOpts = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Kafka Java Heap configuration for broker nodes";
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override Kafka container command";
        };

        args = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override Kafka container arguments";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables";
        };

        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "ConfigMap with extra environment variables";
        };

        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Secret with extra environment variables";
        };

        extraContainerPorts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra containerPorts for the Kafka broker";
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

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Container lifecycle hooks";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements";
        };

        podSecurityContext = mkOption {
          type = types.nullOr t.podSecurityContext;
          default = null;
          description = "Pod security context";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container security context";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount service account token in pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Pod host aliases";
        };

        hostNetwork = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Use host network";
        };

        hostIPC = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Use host IPC";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for broker pods";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for broker pods";
        };

        topologyKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Topology key for affinity";
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
          description = "Node affinity preset";
        };

        affinity = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Affinity for broker pod assignment";
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

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Termination grace period in seconds";
        };

        podManagementPolicy = mkOption {
          type = types.nullOr (types.enum [ "Parallel" "OrderedReady" ]);
          default = null;
          description = "StatefulSet pod management policy";
        };

        minReadySeconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Seconds a pod must be ready before killing the next during update";
        };

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
        };

        runtimeClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Runtime class name";
        };

        enableServiceLinks = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Inject information about services as environment variables";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Alternate scheduler name";
        };

        updateStrategy = mkOption {
          type = types.nullOr t.updateStrategyType;
          default = null;
          description = "Update strategy";
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

        autoscaling = mkOption {
          type = types.nullOr t.autoscalingType;
          default = null;
          description = "VPA/HPA autoscaling configuration";
        };

        pdb = mkOption {
          type = types.nullOr t.pdbType;
          default = null;
          description = "Pod Disruption Budget configuration";
        };

        persistentVolumeClaimRetentionPolicy = mkOption {
          default = null;
          description = "PVC retention policy for the StatefulSet";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Controls if and how PVCs are deleted during the lifecycle of the StatefulSet";
              };

              whenScaled = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when replica count is reduced";
              };

              whenDeleted = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when the StatefulSet is deleted";
              };
            };
          });
        };

        persistence = mkOption {
          default = null;
          description = "Persistence configuration for broker data";
          type = types.nullOr t.persistenceType;
        };

        logPersistence = mkOption {
          default = null;
          description = "Persistence configuration for Kafka logs (separate from data)";
          type = types.nullOr t.persistenceType;
        };
      };
    });
  };
}
