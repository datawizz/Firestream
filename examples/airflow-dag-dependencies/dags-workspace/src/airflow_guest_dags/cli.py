"""Console-script entrypoint for the guest-DAG workspace.

Invoked as `airflow-guest-dag-demo` from the baked guest venv, e.g.
`$FIRESTREAM_DAGS_VENV/bin/airflow-guest-dag-demo`. It imports the
isolation-marker dependency (`cowsay`, absent from Airflow's own venv) and
prints a single final line so a BashOperator can capture it via XCom
(BashOperator pushes only the LAST stdout line).
"""

import sys

import cowsay


def main() -> int:
    # cowsay is the isolation marker: present only in the guest venv.
    print(cowsay.get_output_string("cow", "Firestream guest DAG venv is live"))
    # Final single line -> this is what BashOperator hands to XCom.
    print("airflow-guest-dag-demo: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
