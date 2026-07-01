# PostgreSQL chart options: common / top-level shared parameters.
#
# Covers values.yaml keys at the top level (outside named sections):
# kubeVersion, nameOverride, fullnameOverride, namespaceOverride,
# clusterDomain, extraDeploy, commonLabels, commonAnnotations,
# secretAnnotations, diagnosticMode, postgresqlDataDir,
# postgresqlSharedPreloadLibraries, shmVolume, psp.
#
# Model A: every leaf is nullOr + default = null (sparse override).
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.postgresql = {
    kubeVersion = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Override Kubernetes version";
    };

    nameOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to partially override common.names.fullname";
    };

    fullnameOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to fully override common.names.fullname";
    };

    namespaceOverride = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "String to fully override common.names.namespace";
    };

    clusterDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Kubernetes cluster domain name";
    };

    extraDeploy = mkOption {
      type = types.nullOr (types.listOf types.anything);
      default = null;
      description = "Array of extra objects to deploy with the release";
    };

    commonLabels = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Labels to add to all deployed objects";
    };

    commonAnnotations = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Annotations to add to all deployed objects";
    };

    secretAnnotations = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Add annotations to the secrets";
    };

    diagnosticMode = mkOption {
      default = null;
      description = "Diagnostic mode configuration";
      type = types.nullOr (types.submodule {
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable diagnostic mode";
          };

          command = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Command to override all containers in the statefulset";
          };

          args = mkOption {
            type = types.nullOr (types.listOf types.str);
            default = null;
            description = "Args to override all containers in the statefulset";
          };
        };
      });
    };

    postgresqlDataDir = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "PostgreSQL data dir folder";
    };

    postgresqlSharedPreloadLibraries = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Shared preload libraries (comma-separated list)";
    };

    shmVolume = mkOption {
      default = null;
      description = "Shared memory volume configuration";
      type = types.nullOr (types.submodule {
        options = {
          enabled = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Enable emptyDir volume for /dev/shm for PostgreSQL pod(s)";
          };

          sizeLimit = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Set this to enable a size limit on the shm tmpfs";
          };
        };
      });
    };

    psp = mkOption {
      default = null;
      description = "Pod Security Policy configuration (deprecated, removed in K8s v1.25+)";
      type = types.nullOr (types.submodule {
        options = {
          create = mkOption {
            type = types.nullOr types.bool;
            default = null;
            description = "Whether to create a PodSecurityPolicy";
          };
        };
      });
    };
  };
}
