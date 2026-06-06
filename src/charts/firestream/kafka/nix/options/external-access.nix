# Kafka chart options: `externalAccess.*`.
#
# Per-broker LoadBalancer / NodePort / ClusterIP exposure for controller and
# broker pods. Both sub-blocks share the same shape: a `service.*` submodule
# with port maps, LB IP arrays, annotation arrays, NodePort arrays, etc.
{ lib, ... }:

let
  inherit (lib) mkOption types;

  # One per-role (controller / broker) service block.
  externalServiceSubmodule = types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      type = mkOption {
        type = types.nullOr (types.enum [ "LoadBalancer" "NodePort" "ClusterIP" ]);
        default = null;
        description = "Service type (LoadBalancer | NodePort | ClusterIP)";
      };

      ports = mkOption {
        default = null;
        description = "Service ports";
        type = types.nullOr (types.submodule {
          freeformType = types.attrsOf types.anything;
          options = {
            external = mkOption {
              type = types.nullOr (types.either types.str types.int);
              default = null;
              description = "External Kafka port";
            };
          };
        });
      };

      loadBalancerClass = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Load Balancer class";
      };

      loadBalancerIPs = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Array of LB IPs per Kafka broker";
      };

      loadBalancerNames = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Array of LB hostnames per Kafka broker";
      };

      loadBalancerAnnotations = mkOption {
        # Bitnami: a list of attrs (one per broker); the list-of-attrs shape
        # is unusual for annotation maps but mirrors the upstream values.yaml.
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Array of LB annotations per Kafka broker";
      };

      loadBalancerSourceRanges = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "LB source ranges";
      };

      allocateLoadBalancerNodePorts = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Whether to allocate node ports when service type is LoadBalancer";
      };

      nodePorts = mkOption {
        type = types.nullOr (types.listOf (types.either types.str types.int));
        default = null;
        description = "Array of node ports per Kafka broker";
      };

      externalIPs = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Distinct service host IPs (NodePort)";
      };

      useHostIPs = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Use service host IPs to configure Kafka external listener (NodePort)";
      };

      usePodIPs = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Use MY_POD_IP for external access";
      };

      domain = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Domain or external IP used to configure Kafka external listener";
      };

      publishNotReadyAddresses = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Publish not-ready addresses on the Service";
      };

      labels = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Service labels";
      };

      annotations = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Service annotations";
      };

      extraPorts = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Extra service ports";
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
  };
in {
  options.kafka.externalAccess = mkOption {
    default = null;
    description = "External cluster access to Kafka brokers (LoadBalancer / NodePort / ClusterIP per broker)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable Kubernetes external cluster access to Kafka brokers";
        };

        controller = mkOption {
          default = null;
          description = "Controller-eligible external access";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              forceExpose = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Force exposing controller-eligible nodes even when configured as controller-only";
              };

              service = mkOption {
                type = types.nullOr externalServiceSubmodule;
                default = null;
                description = "Per-broker controller service configuration";
              };
            };
          });
        };

        broker = mkOption {
          default = null;
          description = "Broker-only external access";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              service = mkOption {
                type = types.nullOr externalServiceSubmodule;
                default = null;
                description = "Per-broker external service configuration";
              };
            };
          });
        };
      };
    });
  };
}
