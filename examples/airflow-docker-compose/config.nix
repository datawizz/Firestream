# ---------------------------------------------------------------------------
# YOUR configuration for the Docker Compose example. No cluster, so this is just
# the build-time image customization. Service ports, credentials, and volumes
# come from Firestream's generated compose file (see README.md → "Ports &
# credentials"). The custom DAG is the ./dags folder, wired in via
# firestream/airflow-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  # Reserved for future build-time knobs. The DAG source is ../dags (referenced
  # directly from airflow-image-overrides.nix). Add more .py files there to bake
  # additional DAGs.
}
