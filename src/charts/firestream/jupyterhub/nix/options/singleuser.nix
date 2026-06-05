# JupyterHub chart options: `singleuser.*` (per-user notebook spawner).
#
# The chart materialises `singleuser.*` into KubeSpawner configuration via
# the `hub.configuration` template; many fields are passthrough strings/lists
# (e.g. `profileList`, `extraEnvVars`). Rely on `freeformType` for the long
# tail (cloudMetadata, extraAnnotations, extraLabels, etc.).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.jupyterhub.singleuser = mkOption {
    default = null;
    description = "Per-user notebook (singleuser) spawner configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Singleuser notebook image";
        };

        notebookDir = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Singleuser notebook working directory";
        };

        allowPrivilegeEscalation = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Allow privilege escalation in singleuser containers";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Mount service account token in singleuser pod";
        };

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default container command (defaults to jupyterhub-singleuser)";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables";
        };

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf (types.either types.str types.int));
          default = null;
          description = "Singleuser container ports";
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

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
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
          description = "Singleuser ServiceAccount configuration";
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

        persistence = mkOption {
          default = null;
          description = "Singleuser PVC configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable per-user PVC (else emptyDir)";
              };
              storageClass = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PVC storage class";
              };
              accessModes = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "PVC access modes";
              };
              size = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "PVC size";
              };
            };
          });
        };

        # Free-form list of KubeSpawner profile dicts (display_name,
        # description, kubespawner_override, etc.). Treated as raw passthrough.
        profileList = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "List of JupyterHub profiles (KubeSpawner profile dicts)";
        };

        networkPolicy = mkOption {
          default = null;
          description = "NetworkPolicy configuration for singleuser pods";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Deploy singleuser NetworkPolicy";
              };
              allowExternal = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Don't require server label for connections";
              };
              allowExternalEgress = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow the pod to access any range of port and all destinations";
              };
              allowInterspaceAccess = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow communication between pods in different namespaces";
              };
              allowCloudMetadataAccess = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Allow singleuser pods to access cloud metadata endpoints";
              };
              # The Bitnami chart's defaults type these two as strings (""),
              # not lists, so we mirror that. extraIngress / extraEgress
              # accept the chart's heredoc-style block.
              extraIngress = mkOption {
                type = types.nullOr (types.either types.str (types.listOf types.attrs));
                default = null;
                description = "Extra ingress rules for the NetworkPolicy";
              };
              extraEgress = mkOption {
                type = types.nullOr (types.either types.str (types.listOf types.attrs));
                default = null;
                description = "Extra egress rules for the NetworkPolicy";
              };
              ingressNSMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Namespace labels to allow ingress from";
              };
              ingressNSPodMatchLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Pod labels in matched namespaces to allow ingress from";
              };
            };
          });
        };
      };
    });
  };
}
