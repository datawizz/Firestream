# PostgreSQL chart options: `metrics.*` (postgres-exporter sidecar +
# ServiceMonitor + PrometheusRule).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.postgresql.metrics = mkOption {
    default = null;
    description = "Prometheus exporter (postgres-exporter) configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Start a prometheus exporter";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "postgres-exporter image";
        };

        collectors = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Control enabled collectors";
        };

        customMetrics = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Additional custom metrics";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container security context";
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
          description = "Custom liveness probe";
        };

        customReadinessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom readiness probe";
        };

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe";
        };

        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf types.int);
          default = null;
          description = "postgres-exporter container ports";
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Resource preset";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Resource requirements";
        };

        service = mkOption {
          default = null;
          description = "Metrics service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              ports = mkOption {
                type = types.nullOr (types.attrsOf types.int);
                default = null;
                description = "Metrics service port(s)";
              };

              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Static cluster IP or None";
              };

              sessionAffinity = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Session affinity";
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
                description = "Create ServiceMonitor resource";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace for the ServiceMonitor";
              };

              interval = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Scrape interval";
              };

              scrapeTimeout = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Scrape timeout";
              };

              labels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "ServiceMonitor labels";
              };

              selector = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Prometheus instance selector labels";
              };

              relabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "RelabelConfigs";
              };

              metricRelabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "MetricRelabelConfigs";
              };

              honorLabels = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "honorLabels parameter";
              };

              jobLabel = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Job label name";
              };
            };
          });
        };

        prometheusRule = mkOption {
          default = null;
          description = "PrometheusRule configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a PrometheusRule";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace for the PrometheusRule";
              };

              labels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for the PrometheusRule";
              };

              rules = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "PrometheusRule definitions";
              };
            };
          });
        };
      };
    });
  };
}
