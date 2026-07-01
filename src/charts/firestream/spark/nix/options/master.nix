# Spark chart options: `master.*` (Spark master StatefulSet).
#
# The Bitnami spark chart runs the master as a StatefulSet with podSpec
# knobs typical of Bitnami workloads (security context, affinity, probes,
# PDB, network policy). Typed-and-shallow at the top level with freeform
# passthrough underneath for the long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.spark.master = mkOption {
    default = null;
    description = "Spark master StatefulSet configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Deploy master statefulset";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of an existing ConfigMap with custom configuration for master";
        };

        containerPorts = mkOption {
          default = null;
          description = "Master container ports";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              http = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Master HTTP UI port";
              };
              https = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Master HTTPS UI port";
              };
              cluster = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Master cluster (worker) communication port";
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
          description = "Memory limit for the master daemon";
        };

        configOptions = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Master config options as a string in `-Dx=y` form";
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
          description = "Annotations for master pods";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for master pods";
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
          description = "NetworkPolicy configuration for the master pods";
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
      };
    });
  };
}
