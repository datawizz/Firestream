# Kafka chart options: `metrics.*` (JMX exporter sidecar + ServiceMonitor +
# PrometheusRule). The bundled Kafka exporter is the only Prometheus path
# this chart provides (no separate kafka-exporter sidecar).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.kafka.metrics = mkOption {
    default = null;
    description = "Prometheus JMX exporter, ServiceMonitor, and PrometheusRule configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        jmx = mkOption {
          default = null;
          description = "Prometheus JMX exporter sidecar";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Expose JMX metrics to Prometheus";
              };

              kafkaJmxPort = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "JMX port the exporter scrapes from the Kafka container";
              };

              image = mkOption {
                type = types.nullOr t.imageType;
                default = null;
                description = "JMX exporter image";
              };

              containerSecurityContext = mkOption {
                type = types.nullOr t.containerSecurityContext;
                default = null;
                description = "JMX exporter container security context";
              };

              containerPorts = mkOption {
                default = null;
                description = "JMX exporter container ports";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    metrics = mkOption {
                      type = types.nullOr types.int;
                      default = null;
                      description = "Metrics container port";
                    };
                  };
                });
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

              service = mkOption {
                default = null;
                description = "JMX exporter service configuration";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    ports = mkOption {
                      default = null;
                      description = "JMX exporter service ports";
                      type = types.nullOr (types.submodule {
                        freeformType = types.attrsOf types.anything;
                        options = {
                          metrics = mkOption {
                            type = types.nullOr (types.either types.str types.int);
                            default = null;
                            description = "JMX exporter metrics service port";
                          };
                        };
                      });
                    };

                    clusterIP = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Cluster IP (None for headless)";
                    };

                    sessionAffinity = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Session affinity";
                    };

                    annotations = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Service annotations";
                    };

                    ipFamilies = mkOption {
                      type = types.nullOr (types.listOf types.str);
                      default = null;
                      description = "IP families";
                    };

                    ipFamilyPolicy = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "IP family policy";
                    };
                  };
                });
              };

              whitelistObjectNames = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "JMX object names to expose to JMX stats";
              };

              config = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Configuration file for JMX exporter (templated)";
              };

              existingConfigmap = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Existing ConfigMap with JMX exporter configuration";
              };

              extraRules = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Extra rules to JMX exporter configuration";
              };
            };
          });
        };

        serviceMonitor = mkOption {
          default = null;
          description = "Prometheus Operator ServiceMonitor configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a Prometheus Operator ServiceMonitor";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace in which Prometheus is running";
              };

              path = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Path where JMX exporter serves metrics";
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
                description = "Additional labels for the ServiceMonitor";
              };

              selector = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Prometheus instance selector labels";
              };

              relabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "RelabelConfigs applied before scraping";
              };

              metricRelabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "MetricRelabelConfigs applied before ingestion";
              };

              honorLabels = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "honorLabels parameter";
              };

              jobLabel = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Name of the label on the target service to use as the prometheus job name";
              };
            };
          });
        };

        prometheusRule = mkOption {
          default = null;
          description = "Prometheus Operator PrometheusRule configuration";
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
                description = "Namespace in which Prometheus is running";
              };

              labels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for the PrometheusRule";
              };

              groups = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Prometheus rule groups for Kafka";
              };
            };
          });
        };
      };
    });
  };
}
