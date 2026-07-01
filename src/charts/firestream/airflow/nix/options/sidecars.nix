# Airflow chart options: top-level sidecar container parameters.
#
# Covers values.yaml keys:
#   defaultSidecars - map of chart-provided sidecar containers (e.g. git-sync),
#                     free-form K8s container specs.
#   sidecars        - user-supplied list of extra sidecar containers added to
#                     every Airflow pod.
{ lib, ... }:

let
  inherit (lib) mkOption types;
in {
  options.airflow = {
    defaultSidecars = mkOption {
      type = types.nullOr (types.attrsOf types.anything);
      default = null;
      description = "Default sidecar containers provided by the chart (git sync, etc.)";
    };

    sidecars = mkOption {
      type = types.nullOr (types.listOf types.attrs);
      default = null;
      description = "Add additional sidecar containers to every Airflow pod";
    };
  };
}
