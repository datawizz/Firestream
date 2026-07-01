# ---------------------------------------------------------------------------
# The container-level "make it your own" surface — sibling of
# airflow-overrides.nix (which customizes the Helm CHART). Passed as the LAST
# module to `firestream.lib.<sys>.images.airflow.eval`, yielding a CUSTOM Airflow
# image without forking.
#
# Bakes the DAG files from ../dags into /opt/firestream/airflow/dags. Cloud Build
# pushes the result as <AR>/firestream-airflow:<tag> and airflow-overrides.nix
# points the chart at that same ref, so the cluster runs this image (DAG and all).
#
# Option names mirror src/containers/firestream/airflow/options.nix.
# ---------------------------------------------------------------------------
{ ... }:

{
  config.airflow.vendoredDags = [
    {
      name = "example-dags";
      src = ../dags;
    }
  ];
}
