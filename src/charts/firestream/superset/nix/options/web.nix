# Superset chart options: `web.*` (Superset webserver Deployment).
#
# The web section spans ~470 lines of upstream values: pod-spec knobs,
# probes (liveness/readiness/startup + custom variants), security context,
# autoscaling (HPA + VPA), PDB, networkPolicy, service/ingress, RBAC,
# serviceAccount, metrics (PrometheusRule + ServiceMonitor), volume
# permissions, persistence. Model the most-overridden top-level knobs and
# rely on `freeformType` passthrough for the long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.superset.web = mkOption {
    default = null;
    description = "Superset webserver deployment configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of Superset webserver replicas to deploy";
        };

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf (types.either types.str types.int));
          default = null;
          description = "Container ports for the web pod";
        };

        extraContainerPorts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra ports for the web container(s)";
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

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Container lifecycle hooks";
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

        autoscaling = mkOption {
          type = types.nullOr t.autoscalingType;
          default = null;
          description = "Autoscaling configuration (HPA + VPA)";
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "NetworkPolicy configuration for the web pods";
        };

        service = mkOption {
          default = null;
          description = "Web Service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Web service type";
              };
              ports = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Web service ports";
              };
              nodePorts = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Web service nodePort assignments";
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

        metrics = mkOption {
          default = null;
          description = "Prometheus metrics configuration for the web pod";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable Prometheus metrics";
              };
              serviceMonitor = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "ServiceMonitor configuration for Prometheus Operator";
              };
              prometheusRule = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "PrometheusRule configuration";
              };
            };
          });
        };
      };
    });
  };
}
