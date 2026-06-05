# Superset chart options: `defaultInitContainers.*`.
#
# Two built-in init containers shared by web/worker/beat/flower/init pods:
#   - waitForDB    waits for the metadata database (postgres) to be Ready
#   - waitForRedis waits for the celery broker (redis) to be Ready
#
# Both share the same shape: enabled flag + resources preset/limits +
# containerSecurityContext. Freeform passthrough for the long tail.
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;

  initSubmodule = name: types.submodule {
    freeformType = types.attrsOf types.anything;
    options = {
      enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Enable the ${name} init container";
      };

      resourcesPreset = mkOption {
        type = t.resourcesPreset;
        default = null;
        description = "Resource preset for ${name}";
      };

      resources = mkOption {
        type = types.nullOr t.resourceRequirements;
        default = null;
        description = "Custom resource requirements for ${name}";
      };

      containerSecurityContext = mkOption {
        type = types.nullOr t.containerSecurityContext;
        default = null;
        description = "Container security context for ${name}";
      };
    };
  };
in {
  options.superset.defaultInitContainers = mkOption {
    default = null;
    description = "Default init containers (waitForDB / waitForRedis) shared across superset pods";
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;

      options = {
        waitForDB = mkOption {
          type = types.nullOr (initSubmodule "wait-for-db");
          default = null;
          description = "wait-for-db init container configuration";
        };

        waitForRedis = mkOption {
          type = types.nullOr (initSubmodule "wait-for-redis");
          default = null;
          description = "wait-for-redis init container configuration";
        };
      };
    });
  };
}
