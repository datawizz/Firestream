# Airflow chart options: `metrics.*` (Prometheus StatsD exporter sidecar).
#
# Metrics is a bespoke deployment block, not a generic Airflow component, so its
# keys are declared explicitly (mkComponentType would inject replicaCount / pdb
# / autoscaling that don't exist here). Reuses named pod-spec types where they
# apply. Model A: every leaf nullOr / default = null.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.metrics = mkOption {
    default = { };
    description = "Airflow metrics (StatsD exporter) parameters";
    type = types.submodule {
      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether or not to create a standalone metrics exporter";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "StatsD exporter image";
        };

        configuration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "StatsD exporter mapping configuration";
        };

        existingConfigmap = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing ConfigMap with the StatsD exporter configuration";
        };

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf types.int);
          default = null;
          description = "StatsD exporter container ports (ingest, metrics)";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset; # already nullOr (enum)
          default = null;
          description = "Resource preset for the metrics container";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements for the metrics container";
        };

        podSecurityContext = mkOption {
          type = types.nullOr t.podSecurityContext;
          default = null;
          description = "Pod security context for the metrics pod";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container security context for the metrics container";
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

        startupProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Startup probe configuration";
        };

        customLivenessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom liveness probe (overrides livenessProbe)";
        };

        customReadinessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom readiness probe (overrides readinessProbe)";
        };

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe (overrides startupProbe)";
        };

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Lifecycle hooks for the metrics container";
        };

        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Automount the service account token in the metrics pod";
        };

        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Metrics pod host aliases";
        };

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for the metrics pod";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra annotations for the metrics pod";
        };

        topologyKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Topology key for affinity";
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
          description = "Node affinity preset configuration";
        };

        affinity = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Affinity for the metrics pod assignment";
        };

        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node labels for the metrics pod assignment";
        };

        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name for the metrics pod";
        };

        tolerations = mkOption {
          type = types.nullOr (types.listOf t.tolerationType);
          default = null;
          description = "Tolerations for the metrics pod assignment";
        };

        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf t.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints for the metrics pod";
        };

        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of the k8s scheduler (other than default)";
        };

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Seconds the metrics pod needs to terminate gracefully";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Optionally specify extra list of additional volumes for the metrics pod";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Optionally specify extra list of additional volumeMounts for the metrics container";
        };

        service = mkOption {
          default = null;
          description = "Metrics service parameters";
          type = types.nullOr (types.submodule {
            options = {
              ports = mkOption {
                type = types.nullOr (types.attrsOf types.int);
                default = null;
                description = "StatsD exporter metrics service port(s)";
              };

              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Static cluster IP address or None for headless service";
              };

              sessionAffinity = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Control where client requests go, to the same pod or round-robin";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Annotations for the metrics service";
              };
            };
          });
        };

        serviceMonitor = mkOption {
          default = null;
          description = "Prometheus Operator ServiceMonitor configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a ServiceMonitor for Prometheus Operator";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace in which the ServiceMonitor is created";
              };

              interval = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Interval at which metrics should be scraped";
              };

              scrapeTimeout = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Timeout after which the scrape is ended";
              };

              labels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Extra labels for the ServiceMonitor";
              };

              selector = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Prometheus instance selector labels";
              };

              relabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "RelabelConfigs to apply to samples before scraping";
              };

              metricRelabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "MetricRelabelConfigs to apply to samples before ingestion";
              };

              honorLabels = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Specify honorLabels parameter to add the scrape endpoint";
              };

              jobLabel = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "The name of the label on the target service to use as the job name";
              };
            };
          });
        };

        networkPolicy = mkOption {
          type = types.nullOr t.networkPolicyType;
          default = null;
          description = "Network policy configuration for the metrics pod";
        };
      };
    };
  };
}
