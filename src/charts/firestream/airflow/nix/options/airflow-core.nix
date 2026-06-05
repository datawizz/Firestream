# Airflow chart options: core Airflow deployment-wide parameters.
#
# Covers values.yaml keys:
#   executor, loadExamples, configuration, overrideConfiguration,
#   localSettings, existingConfigmap, and the shared deployment-wide
#   extra env / extra volume keys that sit at the top level of values.yaml:
#   extraEnvVars, extraEnvVarsCM, extraEnvVarsSecret, extraEnvVarsSecrets,
#   extraVolumeMounts, extraVolumes.
#
# Model A: every leaf nullOr + default = null.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow = {
    executor = mkOption {
      type = types.nullOr (types.enum [
        "LocalExecutor"
        "LocalKubernetesExecutor"
        "CeleryExecutor"
        "KubernetesExecutor"
        "CeleryKubernetesExecutor"
      ]);
      default = null;
      description = "Airflow executor. One of LocalExecutor, LocalKubernetesExecutor, CeleryExecutor, KubernetesExecutor, CeleryKubernetesExecutor";
    };

    loadExamples = mkOption {
      type = types.nullOr types.bool;
      default = null;
      description = "Switch to load some Airflow example DAGs";
    };

    configuration = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Specify content for Airflow config file (auto-generated based on other values otherwise)";
    };

    overrideConfiguration = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Add extra configuration to the Airflow config file";
    };

    localSettings = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Specify content for Airflow local settings file (cluster policies)";
    };

    existingConfigmap = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of an existing ConfigMap with the Airflow config file";
    };

    # Deployment-wide extra env / volumes (top level of values.yaml).
    extraEnvVars = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Array with extra environment variables to add to Airflow pods";
    };

    extraEnvVarsCM = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of existing ConfigMap containing extra env vars for Airflow pods";
    };

    extraEnvVarsSecret = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Name of existing Secret containing extra env vars for Airflow pods";
    };

    extraEnvVarsSecrets = mkOption {
      type = types.nullOr (types.listOf types.str);
      default = null;
      description = "List of secrets with extra environment variables for Airflow pods";
    };

    extraVolumeMounts = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Optionally specify extra list of additional volumeMounts for Airflow pods";
    };

    extraVolumes = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Optionally specify extra list of additional volumes for Airflow pods";
    };
  };
}
