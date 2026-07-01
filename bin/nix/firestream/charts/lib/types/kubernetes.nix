# Kubernetes-specific type definitions
# Resources, probes, security contexts, and other K8s primitives.
#
# Model A: every concrete leaf is nullOr-wrapped and defaults to null so that
# an unset option serialises to null and is stripped by the values emitter,
# letting Helm fall back to the chart's own bundled values.yaml defaults.

{ lib, ... }:

let
  inherit (lib) mkOption types;

in {
  # Resource preset enum
  resourcesPreset = types.nullOr (types.enum [
    "none"
    "nano"
    "micro"
    "small"
    "medium"
    "large"
    "xlarge"
    "2xlarge"
  ]);

  # Resource requirements (requests and limits)
  resourceRequirements = types.submodule {
    options = {
      requests = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Resource requests";
        example = { cpu = "100m"; memory = "128Mi"; };
      };

      limits = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Resource limits";
        example = { cpu = "500m"; memory = "512Mi"; };
      };
    };
  };

  # Probe type (liveness, readiness, startup)
  probeType = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable this probe";
      };

      initialDelaySeconds = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Initial delay before probe starts";
      };

      periodSeconds = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "How often to perform the probe";
      };

      timeoutSeconds = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Probe timeout in seconds";
      };

      failureThreshold = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Number of failures before marking unhealthy";
      };

      successThreshold = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Number of successes before marking healthy";
      };
    };
  };

  # Container security context
  containerSecurityContext = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable container security context";
      };

      seLinuxOptions = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "SELinux options";
      };

      runAsUser = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "User ID to run as";
      };

      runAsGroup = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Group ID to run as";
      };

      runAsNonRoot = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Require running as non-root";
      };

      readOnlyRootFilesystem = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Use read-only root filesystem";
      };

      privileged = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Run in privileged mode";
      };

      allowPrivilegeEscalation = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Allow privilege escalation";
      };

      capabilities = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            add = mkOption {
              type = types.nullOr (types.listOf types.str);
              default = null;
              description = "Capabilities to add";
            };

            drop = mkOption {
              type = types.nullOr (types.listOf types.str);
              default = null;
              description = "Capabilities to drop";
            };
          };
        });
        default = null;
        description = "Linux capabilities";
      };

      seccompProfile = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            type = mkOption {
              type = types.nullOr (types.enum [ "RuntimeDefault" "Unconfined" "Localhost" ]);
              default = null;
              description = "Seccomp profile type";
            };

            localhostProfile = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Path to localhost seccomp profile";
            };
          };
        });
        default = null;
        description = "Seccomp profile configuration";
      };
    };
  };

  # Pod security context
  podSecurityContext = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable pod security context";
      };

      fsGroupChangePolicy = mkOption {
        type = types.nullOr (types.enum [ "Always" "OnRootMismatch" ]);
        default = null;
        description = "Filesystem group change policy";
      };

      sysctls = mkOption {
        type = types.nullOr (types.listOf (types.attrsOf types.str));
        default = null;
        description = "Sysctl settings";
      };

      supplementalGroups = mkOption {
        type = types.nullOr (types.listOf types.int);
        default = null;
        description = "Supplemental groups";
      };

      fsGroup = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Filesystem group ID";
      };

      runAsUser = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "User ID to run pod as";
      };

      runAsGroup = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Group ID to run pod as";
      };

      runAsNonRoot = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Require running as non-root";
      };

      seccompProfile = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            type = mkOption {
              type = types.nullOr (types.enum [ "RuntimeDefault" "Unconfined" "Localhost" ]);
              default = null;
              description = "Seccomp profile type";
            };
          };
        });
        default = null;
        description = "Seccomp profile configuration";
      };
    };
  };

  # Update strategy type
  updateStrategyType = types.submodule {
    options = {
      type = mkOption {
        type = types.nullOr (types.enum [ "RollingUpdate" "Recreate" "OnDelete" ]);
        default = null;
        description = "Update strategy type";
      };

      rollingUpdate = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            maxSurge = mkOption {
              type = types.nullOr (types.either types.int types.str);
              default = null;
              description = "Maximum surge during rolling update";
            };

            maxUnavailable = mkOption {
              type = types.nullOr (types.either types.int types.str);
              default = null;
              description = "Maximum unavailable during rolling update";
            };

            partition = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Partition for StatefulSet rolling updates";
            };
          };
        });
        default = null;
        description = "Rolling update configuration";
      };
    };
  };

  # Pod disruption budget type
  pdbType = types.submodule {
    options = {
      create = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Create PodDisruptionBudget";
      };

      minAvailable = mkOption {
        type = types.nullOr (types.either types.int types.str);
        default = null;
        description = "Minimum available pods";
        example = 1;
      };

      maxUnavailable = mkOption {
        type = types.nullOr (types.either types.int types.str);
        default = null;
        description = "Maximum unavailable pods";
        example = "25%";
      };
    };
  };

  # Node affinity preset type
  nodeAffinityPreset = types.submodule {
    options = {
      type = mkOption {
        type = types.nullOr (types.enum [ "" "soft" "hard" ]);
        default = null;
        description = "Node affinity preset type";
      };

      key = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Node label key for affinity";
      };

      values = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Node label values for affinity";
      };
    };
  };

  # Toleration type
  tolerationType = types.submodule {
    options = {
      key = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Toleration key";
      };

      operator = mkOption {
        type = types.nullOr (types.enum [ "Exists" "Equal" ]);
        default = null;
        description = "Toleration operator";
      };

      value = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Toleration value";
      };

      effect = mkOption {
        type = types.nullOr (types.enum [ "" "NoSchedule" "PreferNoSchedule" "NoExecute" ]);
        default = null;
        description = "Toleration effect";
      };

      tolerationSeconds = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Toleration seconds for NoExecute";
      };
    };
  };

  # Topology spread constraint type
  topologySpreadConstraintType = types.submodule {
    options = {
      maxSkew = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Maximum skew between topology domains";
      };

      topologyKey = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Topology key for spreading";
        example = "topology.kubernetes.io/zone";
      };

      whenUnsatisfiable = mkOption {
        type = types.nullOr (types.enum [ "DoNotSchedule" "ScheduleAnyway" ]);
        default = null;
        description = "Action when constraint cannot be satisfied";
      };

      labelSelector = mkOption {
        type = types.nullOr (types.attrsOf types.anything);
        default = null;
        description = "Label selector for pod matching";
      };

      minDomains = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Minimum number of eligible domains";
      };

      nodeAffinityPolicy = mkOption {
        type = types.nullOr (types.enum [ "" "Honor" "Ignore" ]);
        default = null;
        description = "Node affinity policy";
      };

      nodeTaintsPolicy = mkOption {
        type = types.nullOr (types.enum [ "" "Honor" "Ignore" ]);
        default = null;
        description = "Node taints policy";
      };
    };
  };

  # Service type
  serviceType = types.submodule {
    options = {
      type = mkOption {
        type = types.nullOr (types.enum [ "ClusterIP" "NodePort" "LoadBalancer" "ExternalName" ]);
        default = null;
        description = "Service type";
      };

      ports = mkOption {
        type = types.nullOr (types.attrsOf types.int);
        default = null;
        description = "Service ports";
      };

      nodePorts = mkOption {
        type = types.nullOr (types.attrsOf (types.nullOr types.int));
        default = null;
        description = "Node ports (for NodePort type)";
      };

      clusterIP = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Cluster IP address";
      };

      loadBalancerIP = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Load balancer IP";
      };

      loadBalancerSourceRanges = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Load balancer source ranges";
      };

      externalTrafficPolicy = mkOption {
        type = types.nullOr (types.enum [ "" "Cluster" "Local" ]);
        default = null;
        description = "External traffic policy";
      };

      sessionAffinity = mkOption {
        type = types.nullOr (types.enum [ "None" "ClientIP" ]);
        default = null;
        description = "Session affinity";
      };

      sessionAffinityConfig = mkOption {
        type = types.nullOr (types.attrsOf types.anything);
        default = null;
        description = "Session affinity configuration";
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
    };
  };

  # Ingress type
  ingressType = types.submodule {
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable ingress";
      };

      pathType = mkOption {
        type = types.nullOr (types.enum [ "Exact" "Prefix" "ImplementationSpecific" ]);
        default = null;
        description = "Ingress path type";
      };

      apiVersion = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Ingress API version override";
      };

      ingressClassName = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Ingress class name";
      };

      hostname = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Ingress hostname";
      };

      path = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Ingress path";
      };

      annotations = mkOption {
        type = types.nullOr (types.attrsOf types.str);
        default = null;
        description = "Ingress annotations";
      };

      tls = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable TLS";
      };

      selfSigned = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Use self-signed certificate";
      };

      extraHosts = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Extra ingress hosts";
      };

      extraPaths = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Extra ingress paths";
      };

      extraTls = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Extra TLS configuration";
      };

      secrets = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "TLS secrets to create";
      };

      extraRules = mkOption {
        type = types.nullOr (types.listOf types.attrs);
        default = null;
        description = "Extra ingress rules";
      };
    };
  };
}
