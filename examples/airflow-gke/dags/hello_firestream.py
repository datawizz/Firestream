"""A tiny custom DAG, baked into the Firestream Airflow image at build time.

This file is vendored into /opt/firestream/airflow/dags via
`config.airflow.vendoredDags` (see ../firestream/airflow-image-overrides.nix),
so it ships *inside* the container — no ConfigMap, git-sync, or PVC needed. The
scheduler/dag-processor scan it directly. Edit it (or drop more .py files in
this folder) and rebuild the image to change what's baked in.
"""

from __future__ import annotations

import logging
from datetime import datetime

from airflow.sdk import dag, task

log = logging.getLogger("airflow.task")


@dag(
    dag_id="hello_firestream",
    schedule=None,
    start_date=datetime(2024, 1, 1),
    catchup=False,
    tags=["firestream", "example"],
)
def hello_firestream():
    @task
    def say_hello() -> str:
        log.info("Hello from a Firestream-baked DAG!")
        return "ok"

    say_hello()


hello_firestream()
