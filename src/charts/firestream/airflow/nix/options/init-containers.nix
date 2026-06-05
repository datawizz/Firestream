# Airflow chart options: top-level init container parameters.
#
# Covers values.yaml keys:
#   defaultInitContainers  - map of chart-provided init containers (git-clone,
#                            wait-for-* etc.), free-form K8s container specs.
#   initContainers         - user-supplied list of extra init containers added
#                            to every Airflow pod.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow = {
    defaultInitContainers = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Default init containers provided by the chart (git sync, wait-for, prepare-config, etc.)";
    };

    initContainers = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Add additional init containers to every Airflow pod";
    };
  };
}
