# ---------------------------------------------------------------------------
# The container-level "make it your own" surface — sibling of
# airflow-overrides.nix (which customizes the Helm CHART). Passed as the LAST
# module to `firestream.lib.<sys>.images.airflow.eval`, deep-merged on top of
# the Firestream container defaults to yield a CUSTOM Airflow image — without
# forking.
#
# HEADLINE PREFERENCE: declare the guest DAGs' Python dependencies as their own
# uv2nix workspace via `config.airflow.dagWorkspace`. Firestream resolves that
# workspace (../dags-workspace: pyproject.toml + uv.lock) and bakes it into a
# SEPARATE venv at /opt/firestream/airflow/dags-venv — isolated from Airflow's
# OWN closure/venv at /opt/firestream/airflow/venv. It also bakes two env vars
# into every Airflow pod so DAG code can reach that guest venv across a process
# boundary:
#
#     FIRESTREAM_DAGS_VENV   = /opt/firestream/airflow/dags-venv
#     FIRESTREAM_DAGS_PYTHON = /opt/firestream/airflow/dags-venv/bin/python
#
# The workspace's `requires-python` MUST admit python312 (`>=3.12,<3.13` in
# ../dags-workspace/pyproject.toml) — Airflow's interpreter is python312, and
# Firestream's factory throws a legible error at build time if it doesn't. The
# option shape (`{ src; overrides ? null; }`) works exactly like
# `config.airflow.vendoredDags`.
#
# This is the same opt-in discipline as vendoredDags: leave dagWorkspace unset
# and the image is byte-for-byte the stock image.
#
# Option names mirror src/containers/firestream/airflow/options.nix.
# ---------------------------------------------------------------------------
{ ... }:

{
  # THE headline: guest-DAG deps -> a separate, isolated uv2nix venv.
  config.airflow.dagWorkspace = {
    src = ../dags-workspace; # a self-contained uv2nix workspace (pyproject.toml + uv.lock)
    # overrides = ../dags-workspace/overrides.nix;  # optional C-extension fixups
  };

  # dagWorkspace bakes the guest DEPENDENCIES (a venv), not the DAG FILE. So we
  # ALSO bake ../dags/guest_dependencies.py into /opt/firestream/airflow/dags
  # via vendoredDags — exactly like ../airflow-k3s. The two are orthogonal: the
  # DAG file is scanned by Airflow's scheduler; its deps live in the guest venv.
  config.airflow.vendoredDags = [
    {
      name = "guest-dependencies-dag";
      src = ../dags; # a local folder of DAG files (any non-GitHub source works)
    }
  ];
}
