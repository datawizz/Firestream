# ---------------------------------------------------------------------------
# The container-level "make it your own" surface — sibling of
# airflow-overrides.nix (which customizes the Helm CHART). Passed as the LAST
# module to `firestream.lib.<sys>.images.airflow.eval`, deep-merged on top of
# the Firestream container defaults to yield a CUSTOM Airflow image — without
# forking.
#
# Headline preference: bake the DAG files from ../dags into
# /opt/firestream/airflow/dags (Airflow's dags_folder). The resulting image is
# still named `firestream-airflow:3.0.3`, so the (un-repointed) chart picks it
# up once scripts/deploy-local.sh side-loads it. Add more .py files to ../dags
# and rebuild to change what's baked in.
#
# Option names mirror src/containers/firestream/airflow/options.nix.
# ---------------------------------------------------------------------------
{ ... }:

{
  config.airflow.vendoredDags = [
    {
      name = "example-dags";
      src = ../dags; # a local folder of DAG files (any non-GitHub source works)
    }
  ];
}
