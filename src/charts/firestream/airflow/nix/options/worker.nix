# Airflow chart options: `worker.*` (Celery worker StatefulSet).
#
# Composition: shared component base type (mkComponentType) plus worker-only
# StatefulSet extras (podManagementPolicy, extraVolumeClaimTemplates,
# podTemplate).
{ lib, chartTypes, ... }:

let
  t = chartTypes;
  inherit (lib) mkOption types;
in {
  options.airflow.worker = mkOption {
    default = { };
    description = "Airflow Celery worker parameters";
    # Splice the shared component modules (via getSubModules) with worker extras.
    type = types.submodule ((t.mkComponentType { componentName = "worker"; }).getSubModules ++ [
      {
        options = {
          podManagementPolicy = mkOption {
            type = types.nullOr (types.enum [ "OrderedReady" "Parallel" ]);
            default = null;
            description = "Pod management policy for the worker StatefulSet";
          };

          extraVolumeClaimTemplates = mkOption {
            type = types.nullOr (types.listOf types.attrs);
            default = null;
            description = "Optionally specify extra list of volumeClaimTemplates for the worker StatefulSet";
          };

          podTemplate = mkOption {
            type = types.nullOr (types.attrsOf types.anything);
            default = null;
            description = "Pod template used for KubernetesExecutor workers";
          };
        };
      }
    ]);
  };
}
