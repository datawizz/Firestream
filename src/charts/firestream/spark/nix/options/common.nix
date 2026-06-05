# Spark chart options: common / top-level shared parameters.
#
# Covers values.yaml keys at the top level (outside named sections):
# kubeVersion, nameOverride, fullnameOverride, namespaceOverride,
# commonLabels, commonAnnotations, clusterDomain, extraDeploy,
# initScripts, initScriptsCM, initScriptsSecret, hostNetwork, and
# diagnosticMode.
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.spark = {
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

    clusterDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Default Kubernetes cluster domain";
    };

    extraDeploy = mkOption {
      type = types.nullOr (types.listOf types.anything);
      default = null;
      description = "Array of extra objects to deploy with the release";
    };

    initScripts = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Dictionary of init scripts. Evaluated as a template.";
    };

    initScriptsCM = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "ConfigMap with init scripts (overrides initScripts)";
    };

    initScriptsSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Secret with init scripts to be executed at initialization time";
    };

    hostNetwork = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Enable HOST Network for Spark pods";
    };

    diagnosticMode = mkOption {
      default = null;
      description = "Diagnostic mode configuration";
      type = types.nullOr (types.submodule {
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
  };
}
