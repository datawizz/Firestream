# Kafka chart options: `service.*` (Kafka K8s Service + headless services).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.kafka.service = mkOption {
    default = null;
    description = "Kafka K8s service configuration";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        type = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Kubernetes Service type";
        };

        ports = mkOption {
          default = null;
          description = "Service port map";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              client = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kafka svc port for client connections";
              };

              controller = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kafka svc port for controller connections";
              };

              interbroker = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kafka svc port for inter-broker connections";
              };

              external = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kafka svc port for external connections";
              };
            };
          });
        };

        extraPorts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra ports to expose in the Kafka service";
        };

        nodePorts = mkOption {
          default = null;
          description = "NodePort assignments";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              client = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Node port for client connections";
              };

              external = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Node port for external connections";
              };
            };
          });
        };

        sessionAffinity = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Session affinity (ClientIP or None)";
        };

        sessionAffinityConfig = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Additional sessionAffinity settings";
        };

        clusterIP = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Cluster IP for the service";
        };

        loadBalancerIP = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Load Balancer IP";
        };

        loadBalancerClass = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Load Balancer class";
        };

        loadBalancerSourceRanges = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Load Balancer source ranges";
        };

        allocateLoadBalancerNodePorts = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether to allocate node ports when service type is LoadBalancer";
        };

        externalTrafficPolicy = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "External traffic policy";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Additional service annotations";
        };

        headless = mkOption {
          default = null;
          description = "Headless service properties (controller / broker)";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              controller = mkOption {
                default = null;
                description = "Controller-eligible headless service";
                type = types.nullOr (types.submodule {
                  options = {
                    annotations = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Annotations";
                    };
                    labels = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Labels";
                    };
                  };
                });
              };

              broker = mkOption {
                default = null;
                description = "Broker-only headless service";
                type = types.nullOr (types.submodule {
                  options = {
                    annotations = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Annotations";
                    };
                    labels = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Labels";
                    };
                  };
                });
              };

              ipFamilies = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "IP families for the headless service";
              };

              ipFamilyPolicy = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "IP family policy for the headless service";
              };
            };
          });
        };
      };
    });
  };
}
