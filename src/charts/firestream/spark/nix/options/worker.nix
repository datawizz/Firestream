# Spark chart options: `worker.*` (Spark worker StatefulSet).
#
# Shape mirrors master.nix but adds worker-specific knobs: replicaCount,
# memoryLimit / coreLimit / dir / javaOptions, autoscaling (HPA, with
# autoscaling.minReplicas defaulting to "" in the chart so str|int),
# podManagementPolicy. Typed-and-shallow at the top with freeform
# passthrough for the long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.spark.worker = mkOption {
    default = null;
    description = "Spark worker StatefulSet configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Deploy worker resources";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with custom configuration for workers";
        };

        containerPorts = mkOption {
          default = null;
          description = "Worker container ports";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              http = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Worker HTTP UI port";
              };
              https = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Worker HTTPS UI port";
              };
              cluster = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Worker cluster communication port";
              };
            };
          });
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

        extraContainerPorts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra container ports";
        };

        daemonMemoryLimit = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Memory limit for the worker daemon";
        };

        memoryLimit = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Maximum memory the worker is allowed to use";
        };

        coreLimit = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Maximum number of cores that the worker can use";
        };

        dir = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Custom working directory for the application";
        };

        javaOptions = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "JVM options in the form -Dx=y";
        };

        configOptions = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Extra options to configure the worker in the form -Dx=y";
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

        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of spark workers (minimum when autoscaling is enabled)";
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

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for worker pods";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for worker pods";
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
          description = "Affinity for pod assignment";
        };

        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node selector";
        };

        tolerations = mkOption {
          type = types.nullOr (types.listOf t.tolerationType);
          default = null;
          description = "Pod tolerations";
        };

        updateStrategy = mkOption {
          type = types.nullOr t.updateStrategyType;
          default = null;
          description = "Update strategy";
        };

        podManagementPolicy = mkOption {
          type = types.nullOr (types.enum [ "Parallel" "OrderedReady" ]);
          default = null;
          description = "StatefulSet pod management policy";
        };

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
        };

        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf t.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints";
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

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Container lifecycle hooks";
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

        extraVolumeClaimTemplates = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume claim templates";
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

        networkPolicy = mkOption {
          default = null;
          description = "NetworkPolicy configuration for the worker pods";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a NetworkPolicy resource";
              };
              allowExternal = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Don't require client label for connections";
              };
              allowExternalEgress = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow the pod to access any range of port and all destinations";
              };
              extraIngress = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra ingress rules for the NetworkPolicy";
              };
              extraEgress = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra egress rules for the NetworkPolicy";
              };
              ingressNSMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Labels to match to allow traffic from other namespaces";
              };
              ingressNSPodMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Pod labels to match to allow traffic from other namespaces";
              };
            };
          });
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

        # The chart's autoscaling shape is plain HPA (enabled/minReplicas/
        # maxReplicas/targetCPU/targetMemory) and NOT the {vpa,hpa} nested
        # shape that chartTypes.autoscalingType provides. Inline it.
        # minReplicas/targetMemory default to "" upstream, so types.either
        # str int.
        autoscaling = mkOption {
          default = null;
          description = "HPA autoscaling configuration for workers";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable replica autoscaling";
              };
              minReplicas = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Minimum number of worker replicas";
              };
              maxReplicas = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Maximum number of worker replicas";
              };
              targetCPU = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Target CPU utilization percentage";
              };
              targetMemory = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Target memory utilization percentage";
              };
            };
          });
        };
      };
    });
  };
}
