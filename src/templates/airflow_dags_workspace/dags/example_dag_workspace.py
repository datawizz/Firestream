"""Example DAG demonstrating Firestream's guest-DAG venv (options.airflow.dagWorkspace).

When `options.airflow.dagWorkspace` is set to a uv2nix workspace directory
(pyproject.toml + uv.lock), Firestream builds that workspace into a SEPARATE
venv baked at /opt/firestream/airflow/dags-venv (isolated from Airflow's own
venv at /opt/firestream/airflow/venv) and bakes two env vars into every Airflow
pod:

    FIRESTREAM_DAGS_VENV   = /opt/firestream/airflow/dags-venv
    FIRESTREAM_DAGS_PYTHON = /opt/firestream/airflow/dags-venv/bin/python

Airflow's scheduler/dag-processor parse this file with Airflow's OWN
interpreter, so the guest venv is deliberately NOT on their PYTHONPATH (that
would destroy the isolation this feature exists to provide). The two shapes
below cross the process boundary to reach the guest venv.
"""

import os
from datetime import datetime

from airflow import DAG
from airflow.operators.bash import BashOperator
from airflow.decorators import task

with DAG(
    dag_id="example_dag_workspace",
    schedule=None,
    start_date=datetime(2024, 1, 1),
    catchup=False,
    tags=["firestream", "example", "dag-workspace"],
) as dag:

    # --- Shape 1 (RECOMMENDED): BashOperator -> console-script in the guest venv.
    # Separate process, separate interpreter; the guest venv needs NOTHING from
    # Airflow. Params in via CLI args/env; results out via files or stdout.
    #
    # XCom caveat: with do_xcom_push=True (default) BashOperator pushes only the
    # LAST line of stdout to XCom -- not the full stdout. The console-script emits
    # a single final line ("firestream-dag-demo: ok") for exactly this reason; for
    # structured output, write a file or emit one final JSON line.
    run_guest_cli = BashOperator(
        task_id="run_guest_cli",
        bash_command='"$FIRESTREAM_DAGS_VENV/bin/firestream-dag-demo"',
    )

    # --- Shape 2: @task.external_python against the guest interpreter.
    # Two honest tiers (do NOT treat this as "zero cost"):
    #   Tier A (isolated, genuinely cheap): the guest workspace ships NO
    #     apache-airflow. The callable gets serializable args in and returns a
    #     serializable value out (XCom) -- but has NO live Airflow context/macros
    #     / get_current_context(). This example fixture is Tier A.
    #   Tier B (full fidelity): the guest pins apache-airflow==3.0.3 to regain
    #     context + in-callable XCom, at the cost of re-coupling the guest to host
    #     Airflow's full resolution closure and image/build-time blowup. A
    #     deliberate tradeoff -- not free.
    @task.external_python(python=os.environ["FIRESTREAM_DAGS_PYTHON"])
    def run_guest_python() -> str:
        # Body runs under the guest interpreter, so the guest deps import here.
        import cowsay

        return cowsay.get_output_string("cow", "hello from the guest venv")

    run_guest_cli >> run_guest_python()
