from airflow import DAG
from airflow.providers.cncf.kubernetes.operators.kubernetes_pod import KubernetesPodOperator
from datetime import datetime

dag = DAG('pyspark_job', description='Simple tutorial DAG',
          schedule_interval='0 12 * * *',
          start_date=datetime(2022, 3, 20), catchup=False)

kubernetes_min_pod = KubernetesPodOperator(
    namespace='default',
    image="my_pyspark_image:tag",
    cmds=["python","-c"],
    arguments=["from pyspark.sql import SparkSession; spark = SparkSession.builder.appName('airflow-spark').getOrCreate(); ..."],
    labels={"foo": "bar"},
    name="airflow-test-pod",
    task_id="spark-job",
    get_logs=True,
    dag=dag
)
