# Airflow Helm Chart Type Definitions (Model A: sparse-override / null-default)
#
# This module exports all reusable types for the Airflow chart configuration.
#
# Model A design: every concrete leaf option is `nullOr`-wrapped and defaults
# to `null`. A fully-unset component therefore serialises to `{}` (all leaves
# null -> stripped by the values emitter), which is a harmless no-op merge over
# the chart's own bundled values.yaml. A user who sets a single field (e.g.
# `web.replicaCount = 3`) gets exactly `web: { replicaCount: 3 }` in the output.
# This keeps the generated values.yaml a sparse override file and guarantees
# the Phase 5 render-vs-render fidelity check is not broken by baked defaults
# diverging from the chart's true upstream defaults.
#
# Consumed via `import ../types { inherit lib; }` from option modules in
# nix/options/ (Phase 3). The function-arg convention `{ lib, ... }:` is kept
# so that import form works.

{ lib, ... }:

let
  inherit (lib) mkOption types;

  # Import sub-type modules
  kubernetes = import ./kubernetes.nix { inherit lib; };
  image = import ./image.nix { inherit lib; };
  autoscaling = import ./autoscaling.nix { inherit lib; };
  networkPolicy = import ./network-policy.nix { inherit lib; };
  tls = import ./tls.nix { inherit lib; };

in {
  # Re-export all named types
  inherit (kubernetes)
    resourcesPreset
    resourceRequirements
    probeType
    containerSecurityContext
    podSecurityContext
    updateStrategyType
    pdbType
    nodeAffinityPreset
    tolerationType
    topologySpreadConstraintType
    serviceType
    ingressType;

  inherit (image)
    imageType;

  inherit (autoscaling)
    vpaType
    hpaType
    autoscalingType;

  inherit (networkPolicy)
    networkPolicyType;

  inherit (tls)
    tlsType
    certManagerType;

  # Airflow component base type - shared across web, scheduler, worker, etc.
  #
  # This is a function returning a submodule type. Phase 3 passes `componentName`
  # (used only to build per-component descriptions). All emitted leaves default
  # to null (Model A) so an unset component serialises to {}.
  #
  # The legacy default-injecting params (defaultResourcesPreset, defaultReplicas,
  # hasContainerPorts, defaultContainerPorts) are accepted but IGNORED for
  # source compatibility; they no longer influence any default.
  mkComponentType =
    { componentName
    , defaultResourcesPreset ? null   # ignored (Model A)
    , defaultReplicas ? null          # ignored (Model A)
    , hasContainerPorts ? null        # ignored (Model A)
    , defaultContainerPorts ? null    # ignored (Model A)
    }:
    types.submodule {
      options = {
        # Enabled flag (some components are optional)
        enabled = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Enable the ${componentName} component";
        };

        # Replica count
        replicaCount = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Number of ${componentName} replicas";
        };

        # Container ports
        containerPorts = mkOption {
          type = types.nullOr (types.attrsOf types.int);
          default = null;
          description = "Container ports for ${componentName}";
        };

        # Command override
        command = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default container command";
        };

        # Args override
        args = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Override default container arguments";
        };

        # Extra environment variables
        extraEnvVars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra environment variables";
          example = [
            { name = "FOO"; value = "bar"; }
          ];
        };

        # Extra env vars from ConfigMap
        extraEnvVarsCM = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "ConfigMap name with extra environment variables";
        };

        # Extra env vars from Secret
        extraEnvVarsSecret = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Secret name with extra environment variables";
        };

        # Extra env vars from multiple Secrets
        extraEnvVarsSecrets = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "List of secret names with extra environment variables";
        };

        # Resource preset
        resourcesPreset = mkOption {
          type = kubernetes.resourcesPreset;  # already nullOr (enum)
          default = null;
          description = "Resource preset for ${componentName}";
        };

        # Custom resources (overrides preset)
        resources = mkOption {
          type = types.nullOr kubernetes.resourceRequirements;
          default = null;
          description = "Custom resource requirements";
        };

        # Liveness probe
        livenessProbe = mkOption {
          type = types.nullOr kubernetes.probeType;
          default = null;
          description = "Liveness probe configuration";
        };

        # Readiness probe
        readinessProbe = mkOption {
          type = types.nullOr kubernetes.probeType;
          default = null;
          description = "Readiness probe configuration";
        };

        # Startup probe
        startupProbe = mkOption {
          type = types.nullOr kubernetes.probeType;
          default = null;
          description = "Startup probe configuration";
        };

        # Custom probes (raw Kubernetes format)
        customLivenessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom liveness probe (overrides livenessProbe)";
        };

        customReadinessProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom readiness probe (overrides readinessProbe)";
        };

        customStartupProbe = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Custom startup probe (overrides startupProbe)";
        };

        # Pod security context
        podSecurityContext = mkOption {
          type = types.nullOr kubernetes.podSecurityContext;
          default = null;
          description = "Pod security context";
        };

        # Container security context
        containerSecurityContext = mkOption {
          type = types.nullOr kubernetes.containerSecurityContext;
          default = null;
          description = "Container security context";
        };

        # Lifecycle hooks
        lifecycleHooks = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Container lifecycle hooks";
        };

        # Service account token
        automountServiceAccountToken = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Automount service account token";
        };

        # Host aliases
        hostAliases = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Pod host aliases";
        };

        # Pod labels
        podLabels = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra pod labels";
        };

        # Pod annotations
        podAnnotations = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Extra pod annotations";
        };

        # Topology key for affinity
        topologyKey = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Topology key for affinity";
        };

        # Affinity rules
        affinity = mkOption {
          type = types.nullOr (types.attrsOf types.anything);
          default = null;
          description = "Pod affinity rules";
        };

        # Node affinity preset
        nodeAffinityPreset = mkOption {
          type = types.nullOr kubernetes.nodeAffinityPreset;
          default = null;
          description = "Node affinity preset";
        };

        # Pod affinity preset
        podAffinityPreset = mkOption {
          type = types.nullOr (types.enum [ "" "soft" "hard" ]);
          default = null;
          description = "Pod affinity preset";
        };

        # Pod anti-affinity preset
        podAntiAffinityPreset = mkOption {
          type = types.nullOr (types.enum [ "" "soft" "hard" ]);
          default = null;
          description = "Pod anti-affinity preset";
        };

        # Node selector
        nodeSelector = mkOption {
          type = types.nullOr (types.attrsOf types.str);
          default = null;
          description = "Node selector";
        };

        # Tolerations
        tolerations = mkOption {
          type = types.nullOr (types.listOf kubernetes.tolerationType);
          default = null;
          description = "Pod tolerations";
        };

        # Topology spread constraints
        topologySpreadConstraints = mkOption {
          type = types.nullOr (types.listOf kubernetes.topologySpreadConstraintType);
          default = null;
          description = "Topology spread constraints";
        };

        # Priority class
        priorityClassName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Priority class name";
        };

        # Scheduler name
        schedulerName = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Scheduler name";
        };

        # Termination grace period
        terminationGracePeriodSeconds = mkOption {
          type = types.nullOr types.int;
          default = null;
          description = "Termination grace period in seconds";
        };

        # Update strategy
        updateStrategy = mkOption {
          type = types.nullOr kubernetes.updateStrategyType;
          default = null;
          description = "Update strategy";
        };

        # Extra sidecars
        sidecars = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional sidecar containers";
        };

        # Extra init containers
        initContainers = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Additional init containers";
        };

        # Extra volume mounts
        extraVolumeMounts = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volume mounts";
        };

        # Extra volumes
        extraVolumes = mkOption {
          type = types.nullOr (types.listOf types.attrs);
          default = null;
          description = "Extra volumes";
        };

        # Pod disruption budget
        pdb = mkOption {
          type = types.nullOr kubernetes.pdbType;
          default = null;
          description = "Pod disruption budget configuration";
        };

        # Autoscaling
        autoscaling = mkOption {
          type = types.nullOr autoscaling.autoscalingType;
          default = null;
          description = "Autoscaling configuration";
        };

        # Network policy
        networkPolicy = mkOption {
          type = types.nullOr networkPolicy.networkPolicyType;
          default = null;
          description = "Network policy configuration";
        };
      };
    };

  # Git repository type for DAGs/plugins
  gitRepositoryType = types.submodule {
    options = {
      repository = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Git repository URL";
        example = "https://github.com/myuser/myrepo";
      };

      branch = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Git branch to clone";
      };

      name = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Name identifier for this repository";
        example = "my-dags";
      };

      path = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Path within repository to sync";
      };
    };
  };
}
