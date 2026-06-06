# Superset chart options: common / top-level shared parameters.
#
# Covers values.yaml keys at the very top level (outside named sections):
# kubeVersion, apiVersions, nameOverride, fullnameOverride, namespaceOverride,
# clusterDomain, commonLabels, commonAnnotations, extraDeploy, usePasswordFiles,
# diagnosticMode, config, existingConfigmap, loadExamples.
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.superset = {
    kubeVersion = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Override Kubernetes version";
    };

    apiVersions = mkOption {
      type = types.nullOr (types.listOf types.str);
      default = null;
      description = "Override Kubernetes API versions reported by .Capabilities";
    };

    nameOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to partially override common.names.name";
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

    clusterDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Kubernetes cluster domain name";
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

    extraDeploy = mkOption {
      type = types.nullOr (types.listOf types.anything);
      default = null;
      description = "Array of extra objects to deploy with the release";
    };

    usePasswordFiles = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Mount credentials as files instead of using an environment variable";
    };

    diagnosticMode = mkOption {
      default = null;
      description = "Diagnostic mode configuration";
      type = types.nullOr (types.submodule {
        freeformType = types.attrsOf types.anything;
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable diagnostic mode (all probes disabled, container command overridden)";
          };

          command = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Command to override all containers in the chart release";
          };

          args = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Args to override all containers in the chart release";
          };
        };
      });
    };

    config = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap with your custom configuration for Superset";
    };

    existingConfigmap = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap with your custom configuration for Superset";
    };

    loadExamples = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "If true, the Superset examples database will be loaded at startup";
    };
  };
}
