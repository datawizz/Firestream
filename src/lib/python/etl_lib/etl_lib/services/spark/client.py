# import os
# import etl_lib
# from pyspark.sql import SparkSession, Column
# from pyspark.sql import DataFrame
# from pyspark.sql.functions import col, struct, to_json, from_json, decode
from pyspark import SparkContext, SparkConf
from pyspark.sql import SparkSession, DataFrame
# from pyspark.sql.column import Column, _to_java_column




# from pyspark.sql.functions import sha2, concat_ws, bin, hash

# import time
# from datetime import datetime, timedelta
# import json


# from etl_lib.services.kafka.client import KafkaClient




# TODO The key setting is wonky AF
# TODO The schema in schema registry does not appear to get set
# TODO The topic retention time, key col, timestamp, etc should be set when producing records.
# TODO the jars required to run this are sourced from the internet. This should be built from source as part of the project
# TODO The schema registry does not evolve when adding a new field

# TODO https://spark.apache.org/docs/3.1.1/api/python/reference/api/pyspark.sql.streaming.DataStreamWriter.foreachBatch.html
# TODO use the Kubernetes executor for all spark jobs
# TODO build the jars into the dockerfiles served to Kubernetes to minimize spin up and network traffic on executors




import os
from pyspark.sql import SparkSession
import boto3



# TODO this hack is to set the LakeFS credentials, which should be set via the CLI (but cannot be used with MinIO :|  )
os.environ["LAKEFS_ENDPOINT_URL"] = "https://lakefs.default.svc.cluster.local:80/api/v1"
os.environ["LAKEFS_ACCESS_KEY"] = "TODO_CHANGE_ME2"
os.environ["LAKEFS_SECRET_KEY"] = "THIS_IS_A_SECRET_TODO_CHANGE_ME2"




class SparkClient:

    """
    Configures a Spark SQL Context for Batch and Streaming DataSource(s) and DataSink(s)

    Logs actions to a local S3 instance

    """

    def __init__(self, app_name: str, config: dict = {}, **kwargs) -> None:

        self.spark_master = kwargs.get("master") or "local[*]"
        self.CATALOG = kwargs.get("catalog") or "delta"
        self.BRANCH = kwargs.get("branch") or os.environ.get("DEPLOYMENT_MODE", "main")

        #TODO this should be set to the spark master in the K8 cluster
        #self.spark_master = "spark://spark-master-0.spark-headless.default.svc.cluster.local:7077"
        self.app_name = app_name

        # Start with config passed as a parameter
        self.config = config

        # Default to using local S3
        self.storage_location = kwargs.get("storage_location") or "local"



        self.S3_ENDPOINT_URL = os.environ.get("S3_LOCAL_ENDPOINT_URL")
        self.S3_ACCESS_KEY_ID = os.environ.get("S3_LOCAL_ACCESS_KEY_ID")
        self.S3_SECRET_ACCESS_KEY = os.environ.get("S3_LOCAL_SECRET_ACCESS_KEY")
        self.S3_BUCKET_NAME = os.environ.get("S3_LOCAL_BUCKET_NAME")
        self.S3_DEFAULT_REGION = os.environ.get("S3_LOCAL_DEFAULT_REGION")

        self.LAKEFS_ENDPOINT_URL = os.environ.get("LAKEFS_ENDPOINT_URL")
        self.LAKEFS_ACCESS_KEY = os.environ.get("LAKEFS_ACCESS_KEY")
        self.LAKEFS_SECRET_KEY = os.environ.get("LAKEFS_SECRET_KEY")

        self.S3_PATH = f"s3a://{self.S3_BUCKET_NAME}/warehouse"
        # Default logging to local S3 (in cluster or vpc)
        # TODO use opentelemetry to log to a remote logging service
        self.LOG_PATH = f"s3a://{self.S3_BUCKET_NAME}/spark_logs/"


        self.create_logging_dir()

        self.spark_session = self.create_spark_session()
        self.spark_context = self.spark_session.sparkContext

    def set_bucket(self):
        if self.storage_location == "local":
            self.S3_ENDPOINT_URL = os.environ.get("S3_LOCAL_ENDPOINT_URL")
            self.S3_ACCESS_KEY_ID = os.environ.get("S3_LOCAL_ACCESS_KEY_ID")
            self.S3_SECRET_ACCESS_KEY = os.environ.get("S3_LOCAL_SECRET_ACCESS_KEY")
            self.S3_BUCKET_NAME = os.environ.get("S3_LOCAL_BUCKET_NAME")
            self.S3_PATH = f"s3a://{self.S3_BUCKET_NAME}/warehouse"
            self.S3_DEFAULT_REGION = os.environ.get("S3_LOCAL_DEFAULT_REGION")
        else:
            self.S3_ENDPOINT_URL = os.environ.get("S3_CLOUD_ENDPOINT_URL")
            self.S3_ACCESS_KEY_ID = os.environ.get("S3_CLOUD_ACCESS_KEY_ID")
            self.S3_SECRET_ACCESS_KEY = os.environ.get("S3_CLOUD_SECRET_ACCESS_KEY")
            self.S3_BUCKET_NAME = os.environ.get("S3_CLOUD_BUCKET_NAME")
            self.S3_PATH = f"s3a://{self.S3_BUCKET_NAME}/warehouse"
            self.S3_DEFAULT_REGION = os.environ.get("S3_CLOUD_DEFAULT_REGION")

        self.LOG_PATH = f"s3a://{self.S3_BUCKET_NAME}/spark_logs/"

    def create_logging_dir(self):
        s3 = boto3.resource(
            's3',
            endpoint_url=self.S3_ENDPOINT_URL,
            aws_access_key_id=self.S3_ACCESS_KEY_ID,
            aws_secret_access_key=self.S3_SECRET_ACCESS_KEY,
            region_name=self.S3_DEFAULT_REGION
        )

        # Check if bucket exists
        if s3.Bucket(self.S3_BUCKET_NAME) not in s3.buckets.all():
            s3.create_bucket(Bucket=self.S3_BUCKET_NAME)

        try:
            s3.meta.client.head_object(Bucket=self.S3_BUCKET_NAME, Key="spark_logs/")
            print(f"Directory {self.LOG_PATH} exists.")
        except Exception as e:
            print(f"Directory {self.LOG_PATH} does not exist, creating it now.")
            bucket = s3.Bucket(self.S3_BUCKET_NAME)
            bucket.put_object(Key="spark_logs/")

    def config_spark(self):

        catalog = self.CATALOG

        # jars_packages = [
        #     # For Kafka access
        #     "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
        #     "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
        #     "org.apache.kafka:kafka-clients:3.3.1",
        #     "org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.1",
        #     "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1",
        #     "org.apache.hadoop:hadoop-aws:3.3.1",
        #     "org.apache.hadoop:hadoop-common:3.3.1",
        #     "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1"
        # ]



        self.jars_packages = [
            "org.apache.spark:spark-sql-kafka-0-10_2.12:3.4.1",
            "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.4.1",
            "org.apache.kafka:kafka-clients:3.4.1",
            "org.apache.hadoop:hadoop-aws:3.3.1",
            "org.apache.hadoop:hadoop-common:3.3.1",
            "org.apache.spark:spark-hadoop-cloud_2.12:3.4.1",
            "io.delta:delta-core_2.12:2.4.0",
            "io.lakefs:hadoop-lakefs-assembly:0.1.15",
            # "io.delta:delta-core_2.12:2.4.0",
            "org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.3.0",
            "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.70.0"
        ]

        self.jars_packages = ",".join(self.jars_packages)
        print(self.jars_packages)




        self.config.update({
            "spark.jars.packages": self.jars_packages,
            "spark.hadoop.fs.lakefs.impl": "io.lakefs.LakeFSFileSystem",
            # "spark.hadoop.fs.lakefs.api.url": self.LAKEFS_ENDPOINT_URL,
            "spark.hadoop.fs.lakefs.endpoint": self.LAKEFS_ENDPOINT_URL,
            "spark.hadoop.fs.lakefs.connection.ssl.enabled": "false", #TODO ssl on everything! This is just for testing
            "spark.hadoop.fs.lakefs.access.key": self.LAKEFS_ACCESS_KEY,
            "spark.hadoop.fs.lakefs.secret.key": self.LAKEFS_SECRET_KEY,
            "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
            "spark.hadoop.fs.s3a.access.key": self.S3_ACCESS_KEY_ID,
            "spark.hadoop.fs.s3a.secret.key": self.S3_SECRET_ACCESS_KEY,
            "spark.hadoop.fs.s3a.endpoint": self.S3_ENDPOINT_URL,
            "spark.hadoop.fs.s3a.connection.ssl.enabled": "false", # TODO ssl on everything! This is just for testing
            "spark.hadoop.fs.s3a.path.style.access": "true",
            # "spark.hadoop.fs.s3a.committer.name": "directory",
            "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
            "spark.sql.catalog.spark_catalog": "org.apache.spark.sql.delta.catalog.DeltaCatalog",
            # F"spark.sql.catalog.{self.CATALOG}.catalog-impl": "org.apache.spark.sql.delta.catalog.DeltaCatalog",
            "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
            "spark.sql.execution.arrow.pyspark.enabled": "true",
            "spark.sql.session.timeZone": "UTC",
            # "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint", # TODO when deployed to K8 does this persist?
            F"spark.sql.catalog.spark_catalog.warehouse": self.S3_PATH,
            "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint", # TODO when deployed to K8 does this persist, should be be backed by S3?
            F"spark.sql.catalog.{self.CATALOG}.warehouse": self.S3_PATH,
            F"spark.sql.catalog.{self.CATALOG}": "org.apache.iceberg.spark.SparkCatalog",
            F"spark.sql.catalog.{self.CATALOG}.catalog-impl": "org.apache.iceberg.nessie.NessieCatalog",
            F"spark.sql.catalog.{self.CATALOG}.uri": self.NESSIE_SERVER,
            F"spark.sql.catalog.{self.CATALOG}.ref": self.BRANCH,
            F"spark.sql.catalog.{self.CATALOG}.auth_type": "NONE",
            "spark.sql.extensions": "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions",   
            # Set logging to use S3
            # "spark.eventLog.enabled": "true",
            # "spark.eventLog.dir": self.LOG_PATH,
            # "spark.history.fs.logDirectory": F"s3a://{_BUCKET}/",
            "spark.eventLog.rolling.enabled": "true",
            "spark.eventLog.rolling.maxFileSize": "128m",
            "spark.driver.memory": "4g",
            "spark.executor.memory": "4g",
            "spark.offHeap.enabled": "true",
            "spark.memory.offHeap.size": "4g"
        })



    def create_spark_session(self):

        spark = SparkSession.builder.master(self.spark_master).appName(self.app_name)

        self.config_spark()

        for key, value in self.config.items():
            spark.config(key, value)

        spark  = spark.getOrCreate()
        spark.sparkContext.setLogLevel(self.config.get("spark.log.level", "INFO"))
        return spark




    def read(self, format: str, **kwargs) -> DataFrame:
        return self.session.read.format(format).options(**kwargs)
    
    def write(self, df: DataFrame, format: str, **kwargs) -> DataFrame:
        return df.write.format(format).options(**kwargs)
    
    def read_stream(self, format: str, **kwargs) -> DataFrame:
        return self.session.readStream.format(format).options(**kwargs)
    
###########
###TESTS####
###########
from pyspark.sql.types import StructType, StructField, StringType
import pytest

# Test case for read method
def test_read():
    app_name = "TestApp"
    config = {"key": "value"}
    client = SparkClient(app_name=app_name, config=config, storage_location="local")

    data = [("Alice",), ("Bob",)]
    schema = StructType([StructField("name", StringType(), True)])
    df = client.session.createDataFrame(data, schema=schema)
    path = "/tmp/read_test.parquet"
    df.write.mode("overwrite").parquet(path)
    read_df = client.session.read.format("parquet").load(str(path))
    read_df.show()

    # assert read_df.count() == 2

    client.session.stop()

if __name__ == "__main__":
    # pytest.main([__file__])


    test_read()


# from pyspark.sql import SparkSession
# from pyspark.sql.types import StructType, StructField, StringType

# def create_dataframe_from_dict(spark, data_dict):
#     schema = StructType([
#         StructField("name", StringType(), True),
#         StructField("address", StringType(), True),
#         StructField("phone", StringType(), True),
#         StructField("email", StringType(), True)
#     ])
    
#     return spark.createDataFrame(data=[tuple(v for v in record.values()) for record in data_dict.values()], schema=schema)

# # Example usage
# spark = SparkSession.builder.appName("AddressBook").getOrCreate()
# data_dict = {
#     "1": {"name": "John", "address": "123 Main St", "phone": "123-456-7890", "email": "john@example.com"},
#     "2": {"name": "Jane", "address": "456 High St", "phone": "987-654-3210", "email": "jane@example.com"}
# }
# df = create_dataframe_from_dict(spark, data_dict)
# df.show()
        # return self.spark_session.readStream.format(format).options(**kwargs)
    
