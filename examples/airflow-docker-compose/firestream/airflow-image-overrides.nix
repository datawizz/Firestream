# ---------------------------------------------------------------------------
# The container-level "make it your own" surface. Passed as the LAST module to
# `firestream.lib.<sys>.images.airflow.eval`, deep-merged on top of the
# Firestream container defaults to yield a CUSTOM Airflow image — without forking.
#
# Bakes the DAG files from ../dags into /opt/firestream/airflow/dags (Airflow's
# dags_folder). The result is named `firestream-airflow:3.0.3`, exactly what the
# generated compose references, so scripts/up.sh `docker load`s this and
# `docker compose up` runs it. Add more .py files to ../dags and rebuild.
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
