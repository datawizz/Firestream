# Airflow chart options: common / shared top-level parameters.
#
# Covers values.yaml keys: kubeVersion, apiVersions, nameOverride,
# fullnameOverride, namespaceOverride, commonLabels, commonAnnotations,
# clusterDomain, extraDeploy, usePasswordFiles, diagnosticMode.
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow = {
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

    usePasswordFiles = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Mount credentials as files instead of using environment variables";
    };

    diagnosticMode = mkOption {
      default = null;
      description = "Diagnostic mode configuration";
      type = types.nullOr (types.submodule {
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable diagnostic mode (all probes will be disabled and the command will be overridden)";
          };

          command = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Command to override all containers in the deployment";
          };

          args = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Args to override all containers in the deployment";
          };
        };
      });
    };
  };
}
