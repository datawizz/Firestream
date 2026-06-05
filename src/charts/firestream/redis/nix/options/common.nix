# Redis chart options: common / top-level shared parameters.
#
# Covers values.yaml keys at the top level (outside named sections):
# kubeVersion, nameOverride, fullnameOverride, namespaceOverride,
# commonLabels, commonAnnotations, configmapChecksumAnnotations,
# secretChecksumAnnotations, secretAnnotations, clusterDomain,
# extraDeploy, useHostnames, nameResolutionThreshold, nameResolutionTimeout,
# diagnosticMode, commonConfiguration, existingConfigmap, pdb (deprecated).
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.redis = {
    kubeVersion = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Override Kubernetes version";
    };

    nameOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to partially override common.names.fullname";
    };

    fullnameOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to fully override common.names.fullname";
    };

    namespaceOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to fully override common.names.namespace";
    };

    commonLabels = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Labels to add to all deployed objects";
    };

    commonAnnotations = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Annotations to add to all deployed objects";
    };

    configmapChecksumAnnotations = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Enable checksum annotations used to trigger rolling updates when ConfigMap(s) change";
    };

    secretChecksumAnnotations = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Enable checksum annotations used to trigger rolling updates when Secret(s) change";
    };

    secretAnnotations = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Annotations to add to secret";
    };

    clusterDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Kubernetes cluster domain name";
    };

    extraDeploy = mkOption {
      type = types.nullOr (types.listOf types.anything);
      default = null;
      description = "Array of extra objects to deploy with the release";
    };

    useHostnames = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Use hostnames internally when announcing replication";
    };

    nameResolutionThreshold = mkOption {
      type = types.nullOr types.int;
      default = null;
      description = "Failure threshold for internal hostnames resolution";
    };

    nameResolutionTimeout = mkOption {
      type = types.nullOr types.int;
      default = null;
      description = "Timeout seconds between probes for internal hostnames resolution";
    };

    diagnosticMode = mkOption {
      default = null;
      description = "Diagnostic mode configuration";
      type = types.nullOr (types.submodule {
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable diagnostic mode";
          };

          command = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Command to override all containers";
          };

          args = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Args to override all containers";
          };
        };
      });
    };

    commonConfiguration = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Common configuration to be added into the ConfigMap";
    };

    existingConfigmap = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap with custom configuration for Redis nodes";
    };

    # Deprecated top-level pdb (per values.yaml). Kept as a free-form attrset
    # so a user can still pass it through if they need the legacy shape.
    pdb = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "DEPRECATED — use master.pdb / replica.pdb instead";
    };
  };
}
