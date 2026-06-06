# JupyterHub chart options: `hub.*` (main JupyterHub controller Deployment).
#
# The hub section spans ~600 lines of upstream values with deep pod-spec
# knobs (probes, security context, affinity, PDB, networkPolicy, service,
# serviceAccount, metrics, rbac, ingress/service); model the visible
# top-level knobs and rely on `freeformType` passthrough for the long tail.
#
# Bitnami's hub also carries `configuration` (a heredoc-style template
# string) and `services` (an attrset of named API tokens); both are
# untyped passthroughs.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.hub = mkOption {
    default = null;
    description = "JupyterHub controller (hub) deployment configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Hub image (registry/repository/tag/digest/pullPolicy/pullSecrets)";
        };

        baseUrl = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Hub base URL";
        };

        adminUser = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Hub Dummy authenticator admin user";
        };

        password = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Hub Dummy authenticator password (random if empty)";
        };

        services = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "JupyterHub services interacting with the JupyterHub API";
        };

        configuration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Hub configuration file (heredoc template merged into jupyterhub_config.py)";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Configmap with Hub init scripts (replaces templates/hub/configmap.yml)";
        };

        existingSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing secret containing the Hub password";
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

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf (types.either types.str types.int));
          default = null;
          description = "Hub container ports";
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
          description = "Hub ServiceAccount configuration";
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

        rbac = mkOption {
          default = null;
          description = "Hub RBAC configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              create = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create Role and RoleBinding";
              };
              rules = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Custom RBAC rules";
              };
            };
          });
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "NetworkPolicy configuration for the hub pods";
        };

        service = mkOption {
          default = null;
          description = "Hub Service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Hub service type";
              };
              ports = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Hub service ports";
              };
              nodePorts = mkOption {
                type = types.nullOr (types.attrsOf (types.either types.str types.int));
                default = null;
                description = "Hub service nodePort assignments";
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
          description = "Hub Prometheus metrics configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable Hub Prometheus metrics";
              };
              authenticatePrometheus = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Whether to require authentication for the Prometheus /metrics endpoint";
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
