# Import required libraries
from datetime import datetime, timedelta
import time
from airflow import DAG
from airflow.operators.python import PythonOperator

# Define functions for the tasks
def print_start():
    print("Starting the DAG")

def sleep():
    print("Sleeping for 5 seconds")
    time.sleep(5)

def print_end():
    print("Ending the DAG")

# Set default arguments for the DAG
default_args = {
    'owner': 'airflow',
    'depends_on_past': False,
    'email_on_failure': False,
    'email_on_retry': False,
    'retries': 1,
    'retry_delay': timedelta(minutes=1),
}

# Define the DAG
dag = DAG(
    dag_id='example_dag',
    description='A simple example DAG',
    default_args=default_args,
    schedule=timedelta(days=1),
    start_date=datetime(2023, 4, 1),
    catchup=False,
)

# Define the tasks
t1 = PythonOperator(
    task_id='print_start',
    python_callable=print_start,
    dag=dag,
)

t2 = PythonOperator(
    task_id='sleep',
    python_callable=sleep,
    dag=dag,
)

t3 = PythonOperator(
    task_id='print_end',
    python_callable=print_end,
    dag=dag,
)

# Set task dependencies
t1 >> t2 >> t3




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
    dag_id = "example_dag"
    execution_date = "2023-04-01"  # Updated to match the start_date of your DAG for consistency

    # List of task_ids in the order they should be executed
    task_ids = ['print_start', 'sleep', 'print_end']

    # Iterate over the task_ids and execute them in order
    for task_id in task_ids:
        run_airflow_task(dag_id, task_id, execution_date)