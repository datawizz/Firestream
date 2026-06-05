# Kafka chart options: common / top-level shared parameters.
#
# Covers values.yaml keys at the top level (outside named sections):
# kubeVersion, apiVersions, nameOverride, fullnameOverride,
# namespaceOverride, clusterDomain, commonLabels, commonAnnotations,
# extraDeploy, usePasswordFiles, diagnosticMode, plus the top-level
# extraEnvVars / extraVolumes / sidecars / initContainers / dnsPolicy /
# dnsConfig pass-throughs that Bitnami applies chart-wide.
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka = {
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

    clusterDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Default Kubernetes cluster domain";
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

    # Top-level pass-throughs (apply chart-wide unless overridden per-section).
    extraEnvVars = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Extra environment variables to add to Kafka pods (chart-wide)";
    };

    extraEnvVarsCM = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "ConfigMap with extra environment variables (chart-wide)";
    };

    extraEnvVarsSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Secret with extra environment variables (chart-wide)";
    };

    extraVolumes = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Extra volumes for Kafka pods (chart-wide)";
    };

    extraVolumeMounts = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Extra volume mounts for Kafka containers (chart-wide)";
    };

    sidecars = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Additional sidecar containers for Kafka pods (chart-wide)";
    };

    initContainers = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Additional init containers for Kafka pods (chart-wide)";
    };

    dnsPolicy = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "DNS policy for the Kafka pods";
    };

    dnsConfig = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "DNS configuration for the Kafka pods";
    };
  };
}
