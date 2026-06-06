# JupyterHub chart options: `imagePuller.*` (DaemonSet that pre-pulls the
# singleuser image onto every node so user pods cold-start quickly).
#
# Pod-spec knobs only — the puller reuses the `singleuser.image` so it has
# no `image` slot of its own. Rely on `freeformType` passthrough for the
# long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.imagePuller = mkOption {
    default = null;
    description = "DaemonSet that pre-pulls the singleuser image";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Deploy image-puller DaemonSet";
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

        serviceAccount = mkOption {
          default = null;
          description = "Image-puller ServiceAccount configuration";
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
          description = "NetworkPolicy configuration for the image-puller pods";
        };
      };
    });
  };
}
