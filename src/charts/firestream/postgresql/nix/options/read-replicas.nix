# PostgreSQL chart options: `readReplicas.*` (read-only replica StatefulSet,
# used when architecture is `replication`).
#
# Same shape as primary.nix but with replica-specific extras (replicaCount).
# Top-level commonly-touched keys are typed; rest accepted via freeformType.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.readReplicas = mkOption {
    default = null;
    description = "PostgreSQL read-only replica parameters (only used when architecture is replication)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the read replicas database";
        };

        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of PostgreSQL read only replicas";
        };

        extendedConfiguration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Extended PostgreSQL read only replicas configuration";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for read only nodes";
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
          description = "Lifecycle hooks";
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
          description = "Pod Security Context";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container Security Context";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount Service Account token in pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Host aliases";
        };

        hostNetwork = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable host network";
        };

        hostIPC = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable host IPC";
        };

        labels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels for the statefulset";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for pods";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels for the pods";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the pods";
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
          description = "Affinity";
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

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Scheduler name";
        };

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Termination grace period in seconds";
        };

        updateStrategy = mkOption {
          type = types.nullOr t.updateStrategyType;
          default = null;
          description = "Update strategy";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume mounts";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes";
        };

        sidecars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Sidecar containers";
        };

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Init containers";
        };

        pdb = mkOption {
          type = types.nullOr t.pdbType;
          default = null;
          description = "Pod Disruption Budget configuration";
        };

        extraPodSpec = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra PodSpec";
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "Network policy configuration";
        };

        service = mkOption {
          type = types.nullOr t.serviceType;
          default = null;
          description = "Read-only service configuration";
        };

        persistence = mkOption {
          default = null;
          description = "Persistence configuration";
          type = types.nullOr t.persistenceType;
        };

        persistentVolumeClaimRetentionPolicy = mkOption {
          default = null;
          description = "PVC retention policy";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable Persistent volume retention policy";
              };

              whenScaled = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention when replica count is reduced";
              };

              whenDeleted = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention when StatefulSet is deleted";
              };
            };
          });
        };
      };
    });
  };
}
