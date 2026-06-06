# Spark chart options: `service.*` (Spark K8s Service + headless service).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.spark.service = mkOption {
    default = null;
    description = "Spark K8s service configuration";
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
              http = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Spark client port for HTTP";
              };
              https = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Spark client port for HTTPS";
              };
              cluster = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Spark cluster port";
              };
            };
          });
        };

        nodePorts = mkOption {
          default = null;
          description = "NodePort assignments";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              http = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kubernetes web node port for HTTP";
              };
              https = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kubernetes web node port for HTTPS";
              };
              cluster = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Kubernetes cluster node port";
              };
            };
          });
        };

        clusterIP = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Cluster IP for the service";
        };

        loadBalancerIP = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Load balancer IP if service type is LoadBalancer";
        };

        loadBalancerSourceRanges = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Load balancer source ranges";
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

        extraPorts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra service ports";
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

        headless = mkOption {
          default = null;
          description = "Headless service properties";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Annotations for the headless service";
              };
            };
          });
        };
      };
    });
  };
}
