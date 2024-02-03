from datetime import datetime, timedelta
from airflow import DAG
from airflow.providers.cncf.kubernetes.operators.pod import KubernetesPodOperator

# Define default arguments for the DAG
default_args = {
    'owner': 'airflow',
    'depends_on_past': False,
    'email_on_failure': False,
    'email_on_retry': False,
    'retries': 1,
    'retry_delay': timedelta(minutes=5),
}

# Define the DAG
dag = DAG(
    'hello_world_kubernetes',
    default_args=default_args,
    description='A simple hello world DAG using KubernetesPodOperator',
    schedule=timedelta(days=1),
    start_date=datetime(2023, 1, 1),
    catchup=False,
)

# Define the KubernetesPodOperator task
hello_world = KubernetesPodOperator(
    namespace='default',
    image="alpine",
    cmds=["/bin/sh", "-c"],
    arguments=["echo Hello World"],
    name="hello-world",
    task_id="echo_hello_world",
    get_logs=True,
    dag=dag,
)

# Set task in the DAG
hello_world



import subprocess

def run_airflow_task(dag_id, task_id, execution_date):
    """
    Runs an Airflow task using the Airflow CLI.

    Args:
    - dag_id: The ID of the DAG.
    - task_id: The ID of the task.
    - execution_date: The execution date of the task in 'YYYY-MM-DD' format.
    """
    command = f"airflow tasks test {dag_id} {task_id} {execution_date}"
    process = subprocess.run(command, shell=True, capture_output=True, text=True)
    if process.returncode == 0:
        print(f"Successfully ran task {task_id} for dag {dag_id} on {execution_date}")
    else:
        print(f"Error running task {task_id} for dag {dag_id} on {execution_date}")
    print(process.stdout)
    if process.stderr:
        print(process.stderr)

if __name__ == "__main__":
    dag_id = "hello_world_kubernetes"
    execution_date = "2023-04-08"  # This should match the start_date of your DAG

    # Define the tasks in the order they should be executed
    task_ids = ['echo_hello_world']

    # Run the tasks in sequence
    for task_id in task_ids:
        run_airflow_task(dag_id, task_id, execution_date)
