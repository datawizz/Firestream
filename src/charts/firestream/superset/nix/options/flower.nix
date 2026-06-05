# Superset chart options: `flower.*` (Celery flower monitoring UI Deployment).
#
# Flower is the optional web UI for inspecting Celery worker queues.
# `flower.enabled = false` by default in upstream. Has both a `service`
# and (optionally) ingress-like ports. Same pod-spec passthrough pattern
# as worker; we type the most-overridden leaves and rely on freeform
# for the long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.superset.flower = mkOption {
    default = null;
    description = "Celery Flower (monitoring UI) deployment configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the Flower monitoring UI";
        };

        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of Flower replicas";
        };

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf (types.either types.str types.int));
          default = null;
          description = "Container ports for the flower pod";
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
          description = "Custom liveness probe (raw Kubernetes format)";
        };

        customReadinessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom readiness probe (raw Kubernetes format)";
        };

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe (raw Kubernetes format)";
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

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra pod labels";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra pod annotations";
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

        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf t.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints";
        };

        updateStrategy = mkOption {
          type = types.nullOr t.updateStrategyType;
          default = null;
          description = "Update strategy";
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

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional init containers";
        };

        sidecars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional sidecar containers";
        };

        pdb = mkOption {
          type = types.nullOr t.pdbType;
          default = null;
          description = "Pod Disruption Budget configuration";
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "NetworkPolicy configuration for the flower pods";
        };

        service = mkOption {
          default = null;
          description = "Flower Service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Flower service type";
              };
              ports = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Flower service ports";
              };
              nodePorts = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Flower service nodePort assignments";
              };
              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional service annotations";
              };
              extraPorts = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra service ports";
              };
            };
          });
        };
      };
    });
  };
}
