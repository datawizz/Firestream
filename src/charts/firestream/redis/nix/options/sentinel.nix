# Redis chart options: `sentinel.*` (Redis Sentinel sidecar / cluster).
#
# Sentinel is unique to redis (postgresql has no analogue). When enabled it
# runs alongside master/replica pods and exposes both Redis + Sentinel ports
# via the same service. Includes its own image, timing parameters
# (downAfterMilliseconds, failoverTimeout, parallelSyncs), service block (with
# Redis + Sentinel ports), masterService (experimental), and externalAccess
# (LoadBalancer-based external exposure).
#
# Top-level commonly-touched keys are typed; deep / esoteric fields accepted
# via freeformType per Bitnami's deeply-nested style.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.redis.sentinel = mkOption {
    default = null;
    description = "Redis Sentinel configuration parameters";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Use Redis Sentinel on Redis pods (disables master/replica services in favor of a single Redis service exposing both Redis and Sentinel ports)";
        };

        image = mkOption {
          type = types.nullOr t.imageType;
          default = null;
          description = "Redis Sentinel image";
        };

        annotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Additional custom annotations for Redis Sentinel resource";
        };

        masterSet = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Master set name";
        };

        quorum = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Sentinel Quorum";
        };

        getMasterTimeout = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Amount of time to allow before get_sentinel_master_info() times out";
        };

        automateClusterRecovery = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Automate cluster recovery in cases where the last replica is not considered a good replica and Sentinel won't automatically failover to it";
        };

        redisShutdownWaitFailover = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether the Redis master container waits for the failover at shutdown (in addition to the Redis Sentinel container)";
        };

        downAfterMilliseconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Timeout for detecting a Redis node is down";
        };

        failoverTimeout = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Timeout for performing an election failover";
        };

        parallelSyncs = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of replicas that can be reconfigured in parallel to use the new master after a failover";
        };

        configuration = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Configuration for Redis Sentinel nodes";
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

        enableServiceLinks = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether information about services should be injected into pod's environment variable";
        };

        preExecCmds = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Additional commands to run prior to starting Redis Sentinel";
        };

        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables for Redis Sentinel nodes";
        };

        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing ConfigMap containing extra env vars";
        };

        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Name of existing Secret containing extra env vars";
        };

        externalMaster = mkOption {
          default = null;
          description = "Use external master for bootstrapping sentinel";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Use external master for bootstrapping";
              };

              host = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "External master host to bootstrap from";
              };

              port = mkOption {
                type = types.nullOr (types.either types.str types.int);
                default = null;
                description = "Port for Redis service external master host";
              };
            };
          });
        };

        containerPorts = mkOption {
          default = null;
          description = "Container ports for Redis Sentinel nodes";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              sentinel = mkOption {
                type = types.nullOr types.int;
                default = null;
                description = "Container port to open on Redis Sentinel nodes";
              };
            };
          });
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

        persistence = mkOption {
          default = null;
          description = "Persistence configuration for Redis Sentinel nodes";
          type = types.nullOr t.persistenceType;
        };

        persistentVolumeClaimRetentionPolicy = mkOption {
          default = null;
          description = "PVC retention policy for the StatefulSet";
          type = types.nullOr (types.submodule {
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Controls if and how PVCs are deleted during the lifecycle of a StatefulSet";
              };

              whenScaled = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when replica count is reduced";
              };

              whenDeleted = mkOption {
                type = types.nullOr (types.enum [ "Retain" "Delete" ]);
                default = null;
                description = "Volume retention behavior when StatefulSet is deleted";
              };
            };
          });
        };

        resourcesPreset = mkOption {
          type = t.resourcesPreset;
          default = null;
          description = "Set container resources according to one common preset";
        };

        resources = mkOption {
          type = types.nullOr t.resourceRequirements;
          default = null;
          description = "Custom resource requirements";
        };

        containerSecurityContext = mkOption {
          type = types.nullOr t.containerSecurityContext;
          default = null;
          description = "Container Security Context";
        };

        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Lifecycle hooks for the Redis sentinel container";
        };

        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes for the Redis Sentinel";
        };

        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumeMounts for the Redis Sentinel container";
        };

        # service: composite of Redis + Sentinel ports + nodePorts. freeformType
        # because Bitnami merges several headless / extraPorts knobs in here.
        service = mkOption {
          default = null;
          description = "Redis Sentinel service configuration (also configures the master service unless masterService is overridden)";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis Sentinel service type";
              };

              ports = mkOption {
                default = null;
                description = "Service port map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    redis = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Redis service port for Redis";
                    };

                    sentinel = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Redis service port for Sentinel";
                    };
                  };
                });
              };

              nodePorts = mkOption {
                default = null;
                description = "Service nodePort map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    redis = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Node port for Redis";
                    };

                    sentinel = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Node port for Sentinel";
                    };
                  };
                });
              };

              externalTrafficPolicy = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis Sentinel service external traffic policy";
              };

              extraPorts = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra ports to expose (normally used with the `sidecar` value)";
              };

              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis Sentinel service Cluster IP";
              };

              createMaster = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable master service pointing to the current master (experimental)";
              };

              loadBalancerIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis Sentinel service Load Balancer IP";
              };

              loadBalancerClass = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Sentinel service Load Balancer class";
              };

              loadBalancerSourceRanges = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Redis Sentinel service Load Balancer sources";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional custom annotations for Redis Sentinel service";
              };

              sessionAffinity = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Session Affinity for Kubernetes service";
              };

              sessionAffinityConfig = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Additional settings for the sessionAffinity";
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

                    extraPorts = mkOption {
                      type = types.nullOr (types.listOf types.attrs);
                      default = null;
                      description = "Optionally specify extra ports to expose for the headless service";
                    };
                  };
                });
              };
            };
          });
        };

        masterService = mkOption {
          default = null;
          description = "Redis master service parameters (experimental)";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable master service pointing to the current master (experimental)";
              };

              type = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis Sentinel master service type";
              };

              ports = mkOption {
                default = null;
                description = "Service port map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    redis = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Redis service port for Redis";
                    };
                  };
                });
              };

              nodePorts = mkOption {
                default = null;
                description = "Service nodePort map";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    redis = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Node port for Redis";
                    };
                  };
                });
              };

              externalTrafficPolicy = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis master service external traffic policy";
              };

              extraPorts = mkOption {
                type = types.nullOr (types.listOf types.attrs);
                default = null;
                description = "Extra ports to expose";
              };

              clusterIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis master service Cluster IP";
              };

              loadBalancerIP = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Redis master service Load Balancer IP";
              };

              loadBalancerClass = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Master service Load Balancer class";
              };

              loadBalancerSourceRanges = mkOption {
                type = types.nullOr (types.listOf types.str);
                default = null;
                description = "Redis master service Load Balancer sources";
              };

              annotations = mkOption {
                type = types.nullOr (types.attrsOf types.str);
                default = null;
                description = "Additional custom annotations for Redis master service";
              };

              sessionAffinity = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Session Affinity for Kubernetes service";
              };

              sessionAffinityConfig = mkOption {
                type = types.nullOr (types.attrsOf types.anything);
                default = null;
                description = "Additional settings for the sessionAffinity";
              };
            };
          });
        };

        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr (types.either types.str types.int);
          default = null;
          description = "Termination grace period in seconds";
        };

        extraPodSpec = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Extra PodSpec for the Redis Sentinel pod(s)";
        };

        # externalAccess: LoadBalancer-based external exposure for sentinels.
        externalAccess = mkOption {
          default = null;
          description = "External access configuration for sentinel";
          type = types.nullOr (types.submodule {
            freeformType = types.attrsOf types.anything;
            options = {
              enabled = mkOption {
                type = types.nullOr types.bool;
                default = null;
                description = "Enable external access to the Redis";
              };

              service = mkOption {
                default = null;
                description = "External access service configuration";
                type = types.nullOr (types.submodule {
                  freeformType = types.attrsOf types.anything;
                  options = {
                    loadBalancerIPAnnotaion = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Name of annotation to specify fixed IP for service";
                    };

                    type = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Type for the services used to expose every Pod";
                    };

                    redisPort = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Port for the services used to expose redis-server";
                    };

                    sentinelPort = mkOption {
                      type = types.nullOr (types.either types.str types.int);
                      default = null;
                      description = "Port for the services used to expose redis-sentinel";
                    };

                    loadBalancerIP = mkOption {
                      type = types.nullOr (types.listOf types.str);
                      default = null;
                      description = "Array of load balancer IPs for each Redis node";
                    };

                    loadBalancerClass = mkOption {
                      type = types.nullOr types.str;
                      default = null;
                      description = "Load Balancer class if service type is LoadBalancer";
                    };

                    loadBalancerSourceRanges = mkOption {
                      type = types.nullOr (types.listOf types.str);
                      default = null;
                      description = "Service Load Balancer sources";
                    };

                    annotations = mkOption {
                      type = types.nullOr (types.attrsOf types.str);
                      default = null;
                      description = "Annotations to add to the services used to expose every Pod";
                    };
                  };
                });
              };
            };
          });
        };
      };
    });
  };
}
