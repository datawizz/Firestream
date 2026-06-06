# Spark chart options: `metrics.*` (Prometheus annotations + PodMonitor +
# PrometheusRule). The spark chart does NOT ship a dedicated exporter
# container — Spark's own driver/master/worker expose /metrics over their
# UI ports, and the chart sets prometheus.io/scrape annotations and a
# PodMonitor pointing at those ports.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.spark.metrics = mkOption {
    default = null;
    description = "Prometheus metrics configuration (annotations + PodMonitor + PrometheusRule)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Start a side-car prometheus exporter / expose metrics";
        };

        masterAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the Prometheus metrics on master pods (templated)";
        };

        workerAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for the Prometheus metrics on worker pods (templated)";
        };

        podMonitor = mkOption {
          default = null;
          description = "PodMonitor configuration for Prometheus Operator";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a PodMonitor resource";
              };

              extraMetricsEndpoints = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Add metrics endpoints for monitoring jobs running in workers";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace in which the podMonitor resource will be created";
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

              additionalLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for PodMonitor discovery";
              };
            };
          });
        };

        prometheusRule = mkOption {
          default = null;
          description = "PrometheusRule configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a PrometheusRule resource";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace in which the PrometheusRule resource will be created";
              };

              additionalLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for PrometheusRule discovery";
              };

              rules = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Custom Prometheus rules";
              };
            };
          });
        };
      };
    });
  };
}
