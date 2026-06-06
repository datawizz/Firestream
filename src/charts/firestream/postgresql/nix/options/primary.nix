# PostgreSQL chart options: `primary.*` (the primary StatefulSet).
#
# The Bitnami `primary:` block is a hand-rolled mix of generic pod-spec keys
# (probes / resources / security context / scheduling) PLUS a lot of bespoke
# PostgreSQL keys (configuration, pgHbaConfiguration, initdb scripts,
# persistence, networkPolicy, service, standby, pdb). We declare the
# commonly-touched top-level keys typed-and-shallow, and rely on the submodule's
# freeformType for the deeper / less-frequently-touched fields.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.primary = mkOption {
    default = null;
    description = "PostgreSQL primary StatefulSet parameters";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        name = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the primary database";
        };

        configuration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "PostgreSQL Primary main configuration to be injected as ConfigMap";
        };

        pgHbaConfiguration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "PostgreSQL Primary client authentication configuration";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with PostgreSQL Primary configuration";
        };

        extendedConfiguration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Extended PostgreSQL Primary configuration";
        };

        existingExtendedConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with PostgreSQL Primary extended configuration";
        };

        initdb = mkOption {
          default = null;
          description = "Initdb configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              args = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PostgreSQL initdb extra arguments";
              };

              postgresqlWalDir = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Specify a custom location for the PostgreSQL transaction log";
              };

              scripts = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Dictionary of initdb scripts";
              };

              scriptsConfigMap = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "ConfigMap with scripts to be run at first boot";
              };

              scriptsSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Secret with scripts to be run at first boot";
              };

              user = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PostgreSQL username to execute the initdb scripts";
              };

              password = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PostgreSQL password to execute the initdb scripts";
              };
            };
          });
        };

        preInitDb = mkOption {
          default = null;
          description = "Pre-init script configuration";
          type = types.nullOr (types.submodule {
            options = {
              scripts = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Dictionary of pre-init scripts";
              };

              scriptsConfigMap = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "ConfigMap with pre-init scripts";
              };

              scriptsSecret = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Secret with pre-init scripts";
              };
            };
          });
        };

        standby = mkOption {
          default = null;
          description = "Standby (cross-cluster replication) configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable standby mode for cross-cluster replication";
              };

              primaryHost = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Host of replication primary in the other cluster";
              };

              primaryPort = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Port of replication primary in the other cluster";
              };
            };
          });
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for PostgreSQL Primary nodes";
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
          description = "Lifecycle hooks for the PostgreSQL Primary container";
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

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount Service Account token in pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "PostgreSQL primary pods host aliases";
        };

        hostNetwork = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable host network for PostgreSQL pod";
        };

        hostIPC = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable host IPC for PostgreSQL pod";
        };

        labels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels to add to the statefulset";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for PostgreSQL primary pods";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Labels to add to the pods";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations to add to the pods";
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
          description = "Affinity for PostgreSQL primary pods assignment";
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
          # Bitnami: "" by default but can be int.
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
          description = "Additional sidecar containers";
        };

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional init containers";
        };

        pdb = mkOption {
          type = types.nullOr t.pdbType;
          default = null;
          description = "Pod Disruption Budget configuration";
        };

        extraPodSpec = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra PodSpec for the PostgreSQL Primary pod(s)";
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "Network policy configuration";
        };

        service = mkOption {
          type = types.nullOr t.serviceType;
          default = null;
          description = "Primary service configuration";
        };

        persistence = mkOption {
          default = null;
          description = "Persistence configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable persistence using PVC";
              };

              volumeName = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name to assign the volume";
              };

              existingClaim = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of an existing PVC to use";
              };

              mountPath = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Path where the volume will be mounted";
              };

              subPath = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Subdirectory of the volume to mount to";
              };

              storageClass = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PVC Storage Class";
              };

              accessModes = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "PVC Access Modes";
              };

              size = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PVC Storage Request size";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Annotations for the PVC";
              };

              labels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Labels for the PVC";
              };

              selector = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Selector to match an existing Persistent Volume";
              };

              dataSource = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Custom PVC data source";
              };
            };
          });
        };

        persistentVolumeClaimRetentionPolicy = mkOption {
          default = null;
          description = "PVC retention policy for the StatefulSet";
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
      };
    });
  };
}
