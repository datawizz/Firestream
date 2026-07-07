"""Guest-DAG package resolved via Firestream's dagWorkspace uv2nix seam.

Its single dependency (`cowsay`) is deliberately absent from Airflow's own
closure, so importing it proves the guest venv at
/opt/firestream/airflow/dags-venv is isolated from Airflow's venv at
/opt/firestream/airflow/venv.
"""

__version__ = "0.1.0"
