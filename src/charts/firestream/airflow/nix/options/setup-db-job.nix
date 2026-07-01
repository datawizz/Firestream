# Airflow chart options: `setupDBJob.*` (database setup/migration Job).
#
# This is a Kubernetes Job, not a long-running component, so it intentionally
# does NOT use mkComponentType (which would inject component-only keys such as
# replicaCount/pdb/autoscaling/networkPolicy that have no meaning for a Job).
# Instead every one of its 30 values.yaml keys is declared explicitly, reusing
# the named pod-spec types where they apply. Model A: all leaves nullOr/null.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.setupDBJob = mkOption {
    default = { };
    description = "Airflow database setup/migration Job parameters";
    type = types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the database setup Job";
        };

        backoffLimit = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of retries before considering the Job as failed";
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

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Array with extra environment variables to add to the Job";
        };

        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing ConfigMap containing extra env vars for the Job";
        };

        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing Secret containing extra env vars for the Job";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset; # already nullOr (enum)
          default = null;
          description = "Resource preset for the Job container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements for the Job container";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Automount the service account token in the Job pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Job pod host aliases";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Add annotations to the Job";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for the Job pod";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra annotations for the Job pod";
        };

        topologyKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Topology key for affinity";
        };

        affinity = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Affinity for the Job pod assignment";
        };

        nodeAffinityPreset = mkOption {
          type = types.nullOr t.nodeAffinityPreset;
          default = null;
          description = "Node affinity preset configuration";
        };

        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node labels for the Job pod assignment";
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

        tolerations = mkOption {
          type = types.nullOr (types.listOf t.tolerationType);
          default = null;
          description = "Tolerations for the Job pod assignment";
        };

        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf t.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints for the Job pod";
        };

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name for the Job pod";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the k8s scheduler (other than default)";
        };

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Seconds the Job pod needs to terminate gracefully";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Optionally specify extra list of additional volumes for the Job pod";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Optionally specify extra list of additional volumeMounts for the Job container";
        };

        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Add additional init containers to the Job pod";
        };
      };
    };
  };
}
