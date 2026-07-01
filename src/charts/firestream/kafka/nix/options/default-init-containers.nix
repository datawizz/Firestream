# Kafka chart options: `defaultInitContainers.*`.
#
# Kafka bundles three default init containers - volume-permissions (chown the
# data volume), prepare-config (template the server.properties), and
# auto-discovery (query the K8s API for LB IPs / NodePorts when externalAccess
# is on). All three are typed at the top-level slots; their inner shape is
# common (image / containerSecurityContext / resources*) and we expose the
# whole submodule as freeform underneath so unknown sub-keys survive.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;

  # Shared shape for the three init containers below. Each has an image,
  # a containerSecurityContext (with the dual-typed runAsUser="auto"|int
  # quirk on volume-permissions), and resources*.
  initContainerSubmodule = types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable this init container";
      };

      image = mkOption {
        type = types.nullOr t.imageType;
        default = null;
        description = "Init container image";
      };

      containerSecurityContext = mkOption {
        # Bitnami's containerSecurityContext here is intentionally permissive:
        # only seLinuxOptions + a handful of runtime flags are surfaced; the
        # rest pass through. runAsUser is dual-typed (int UID or the literal
        # "auto" for OpenShift).
        default = null;
        description = "Init container security context";
        type = types.nullOr (types.submodule {
          freeformType = types.attrsOf types.anything;
          options = {
            enabled = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "Enable the security context block";
            };

            seLinuxOptions = mkOption {
              type = types.nullOr (types.attrsOf types.anything);
              default = null;
              description = "SELinux options";
            };

            runAsUser = mkOption {
              type = types.nullOr (types.either types.str types.int);
              default = null;
              description = "runAsUser (use 'auto' for OpenShift)";
            };

            runAsGroup = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "runAsGroup";
            };

            runAsNonRoot = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "runAsNonRoot";
            };

            readOnlyRootFilesystem = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "readOnlyRootFilesystem";
            };

            privileged = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "privileged";
            };

            allowPrivilegeEscalation = mkOption {
              type = types.nullOr types.bool;
              default = null;
              description = "allowPrivilegeEscalation";
            };

            capabilities = mkOption {
              type = types.nullOr (types.attrsOf types.anything);
              default = null;
              description = "Linux capabilities to add/drop";
            };

            seccompProfile = mkOption {
              type = types.nullOr (types.attrsOf types.anything);
              default = null;
              description = "Seccomp profile";
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

      extraInit = mkOption {
        # prepareConfig-specific but harmless on the others.
        type = types.nullOr types.str;
        default = null;
        description = "Additional content for the init script (templated)";
      };
    };
  };
in {
  options.kafka.defaultInitContainers = mkOption {
    default = null;
    description = "Bitnami default init containers (volume-permissions, prepare-config, auto-discovery)";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        volumePermissions = mkOption {
          type = types.nullOr initContainerSubmodule;
          default = null;
          description = "Init container that chowns the data volume mountpoint";
        };

        prepareConfig = mkOption {
          type = types.nullOr initContainerSubmodule;
          default = null;
          description = "Init container that prepares the Kafka configuration files";
        };

        autoDiscovery = mkOption {
          type = types.nullOr initContainerSubmodule;
          default = null;
          description = "Init container that auto-detects external IPs/ports via the K8s API";
        };
      };
    });
  };
}
