# Redis chart options: `metrics.*` (redis-exporter sidecar + ServiceMonitor +
# PodMonitor + PrometheusRule).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.metrics = mkOption {
    default = null;
    description = "Prometheus exporter (redis-exporter) configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Start a sidecar prometheus exporter to expose Redis metrics";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "redis-exporter image";
        };

        containerPorts = mkOption {
          default = null;
          description = "Metrics container ports";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              http = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Metrics HTTP container port";
              };
            };
          });
        };

        startupProbe = mkOption {
          type = types.nullOr t.probeType;
          default = null;
          description = "Startup probe configuration";
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

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe";
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

        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default metrics container init command";
        };

        redisTargetHost = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "A way to specify an alternative Redis hostname";
        };

        extraArgs = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra arguments for Redis exporter";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for Redis exporter";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container Security Context for the metrics sidecar";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes for the Redis metrics sidecar";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume mounts for the Redis metrics sidecar";
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

        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra labels for Redis exporter pods";
        };

        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Annotations for Redis exporter pods";
        };

        service = mkOption {
          default = null;
          description = "Metrics service configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create Service resource(s) for scraping metrics";
              };

              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis exporter service type";
              };

              ports = mkOption {
                default = null;
                description = "Service port map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    http = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Redis exporter service port";
                    };
                  };
                });
              };

              externalTrafficPolicy = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis exporter service external traffic policy";
              };

              extraPorts = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra ports to expose";
              };

              loadBalancerIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis exporter service Load Balancer IP";
              };

              loadBalancerClass = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Exporter service Load Balancer class";
              };

              loadBalancerSourceRanges = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Redis exporter service Load Balancer sources";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional custom annotations for Redis exporter service";
              };

              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis exporter service Cluster IP";
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
                description = "Create ServiceMonitor resource(s) for scraping metrics";
              };

              port = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "The service port to scrape metrics from";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace for the ServiceMonitor";
              };

              tlsConfig = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "TLS configuration used for scrape endpoints used by Prometheus";
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

              relabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "RelabelConfigs to apply to samples before scraping";
              };

              relabellings = mkOption {
                # Bitnami DEPRECATED key but still present in values.yaml.
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "DEPRECATED: Use `relabelings` instead";
              };

              metricRelabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "MetricRelabelConfigs to apply to samples before ingestion";
              };

              honorLabels = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "honorLabels parameter";
              };

              additionalLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for the ServiceMonitor";
              };

              podTargetLabels = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Labels from the Kubernetes pod to be transferred to the created metrics";
              };

              sampleLimit = mkOption {
                # Bitnami: false by default; can be int.
                type = types.nullOr (types.either types.bool types.int);
                default = null;
                description = "Limit of how many samples should be scraped from every Pod";
              };

              targetLimit = mkOption {
                type = types.nullOr (types.either types.bool types.int);
                default = null;
                description = "Limit of how many targets should be scraped";
              };

              additionalEndpoints = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Additional endpoints to scrape (e.g sentinel)";
              };
            };
          });
        };

        podMonitor = mkOption {
          default = null;
          description = "Prometheus Operator PodMonitor configuration";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create PodMonitor resource(s) for scraping metrics";
              };

              port = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "The pod port to scrape metrics from";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace for the PodMonitor";
              };

              tlsConfig = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "TLS configuration used for scrape endpoints used by Prometheus";
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

              relabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "RelabelConfigs to apply to samples before scraping";
              };

              relabellings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "DEPRECATED: Use `relabelings` instead";
              };

              metricRelabelings = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "MetricRelabelConfigs to apply to samples before ingestion";
              };

              honorLabels = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "honorLabels parameter";
              };

              additionalLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for the PodMonitor";
              };

              podTargetLabels = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Labels from the Kubernetes pod to be transferred to the created metrics";
              };

              sampleLimit = mkOption {
                type = types.nullOr (types.either types.bool types.int);
                default = null;
                description = "Limit of how many samples should be scraped from every Pod";
              };

              targetLimit = mkOption {
                type = types.nullOr (types.either types.bool types.int);
                default = null;
                description = "Limit of how many targets should be scraped";
              };

              additionalEndpoints = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Additional endpoints to scrape (e.g sentinel)";
              };
            };
          });
        };

        prometheusRule = mkOption {
          default = null;
          description = "Prometheus Rule configuration";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Create a custom prometheusRule Resource";
              };

              namespace = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Namespace for the PrometheusRule";
              };

              additionalLabels = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional labels for the prometheusRule";
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
