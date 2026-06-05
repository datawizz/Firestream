# JupyterHub chart options: `proxy.*` (configurable-http-proxy Deployment).
#
# The proxy is the front-door for browser traffic; the chart wires its
# `secretToken` either from `proxy.secretToken` or a random secret. Pod-spec
# knobs mirror the hub section; rely on `freeformType` passthrough for the
# long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.proxy = mkOption {
    default = null;
    description = "configurable-http-proxy deployment configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Proxy image (registry/repository/tag/digest/pullPolicy/pullSecrets)";
        };

        secretToken = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Hex-encoded shared secret between hub and proxy (random if empty)";
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

        containerPort = mkOption {
          type = types.nullOr (types.attrsOf (types.either types.str types.int));
          default = null;
          description = "Proxy container ports";
        };

        startupProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Startup probe configuration";
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

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe";
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

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container security context";
        };

        podSecurityContext = mkOption {
          type = types.nullOr t.podSecurityContext;
          default = null;
          description = "Pod security context";
        };

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Container lifecycle hooks";
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

        serviceAccount = mkOption {
          default = null;
          description = "Proxy ServiceAccount configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              create = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable ServiceAccount creation";
              };
              name = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the service account to use";
              };
              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Annotations for the ServiceAccount";
              };
              automountServiceAccountToken = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow auto-mount of ServiceAccountToken";
              };
            };
          });
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "NetworkPolicy configuration for the proxy pods";
        };

        service = mkOption {
          default = null;
          description = "Proxy Service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Proxy service type";
              };
              ports = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Proxy service ports";
              };
              nodePorts = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Proxy service nodePort assignments";
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
              loadBalancerIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Load balancer IP";
              };
              loadBalancerSourceRanges = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Load balancer source ranges";
              };
              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Cluster IP";
              };
              externalTrafficPolicy = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "External traffic policy";
              };
              sessionAffinity = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Session affinity (ClientIP or None)";
              };
              sessionAffinityConfig = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Additional sessionAffinity settings";
              };
            };
          });
        };

        ingress = mkOption {
          type = types.nullOr t.ingressType;
          default = null;
          description = "Proxy Ingress configuration";
        };

        metrics = mkOption {
          default = null;
          description = "Proxy Prometheus metrics configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable Proxy Prometheus metrics";
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
