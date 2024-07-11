from datetime import datetime
from airflow import DAG
from airflow.operators.dummy import DummyOperator
from your_module import SparkKubernetesOperator

dag = DAG(
    dag_id='spark_kubernetes_operator_dag',
    start_date=datetime(2023, 4, 8),
    schedule_interval=None,
)

start = DummyOperator(task_id='start', dag=dag)

spark_job = SparkKubernetesOperator(
    task_id='spark_job',
    spark_conf={
        'spark.executor.instances': '2',
        'spark.executor.memory': '2g',
    },
    spark_application_file='/path/to/your/spark_application.yaml',
    dag=dag,
)

end = DummyOperator(task_id='end', dag=dag)

start >> spark_job >> end
