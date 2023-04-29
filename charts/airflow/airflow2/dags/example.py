# Import required libraries
from datetime import datetime, timedelta
from airflow import DAG
from airflow.operators.python_operator import PythonOperator

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
    schedule_interval=timedelta(days=1),
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
