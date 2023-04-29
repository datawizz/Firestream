






from pyspark import SparkConf
from pyspark.sql import SparkSession
import os
#from delta.tables import *



PATH = F"s3a://{os.environ.get('S3_BUCKET_NAME')}/example/data/delta-table3"


def spark_config():


    config = {
        # "spark.kubernetes.namespace": "spark",
        # "spark.kubernetes.container.image": "itayb/spark:3.1.1-hadoop-3.2.0-aws",
        # "spark.executor.instances": "2",
        # "spark.executor.memory": "1g",
        # "spark.executor.cores": "1",
        # "spark.driver.blockManager.port": "7777",
        # "spark.driver.port": "2222",
        # "spark.driver.host": "jupyter.spark.svc.cluster.local",
        # "spark.driver.bindAddress": "0.0.0.0",
        "spark.hadoop.fs.s3a.endpoint": os.environ.get("S3_ENDPOINT_URL"),
        "spark.hadoop.fs.s3a.access.key": os.environ.get("S3_ACCESS_KEY_ID"),
        "spark.hadoop.fs.s3a.secret.key": os.environ.get("S3_SECRET_ACCESS_KEY"),
        "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
        "spark.hadoop.fs.s3a.path.style.access": "true",
        "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
        "spark.hadoop.com.amazonaws.services.s3.enableV4": "true",
        # "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.AnonymousAWSCredentialsProvider",
        "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
        "spark.jars.packages": "org.apache.hadoop:hadoop-aws:3.3.1,org.apache.hadoop:hadoop-common:3.3.1,org.apache.spark:spark-hadoop-cloud_2.12:3.3.1,io.delta:delta-core_2.12:2.2.0",
        # "spark.hadoop.parquet.summary.metadata.level": "false",
        "spark.sql.parquet.mergeSchema": "false",
        "spark.sql.parquet.filterPushdown": "true",
        "spark.sql.hive.metastorePartitionPruning": "true",
        "spark.hadoop.fs.s3a.committer.name": "directory",
        "spark.sql.sources.commitProtocolClass": "org.apache.spark.internal.io.cloud.PathOutputCommitProtocol",
        "spark.sql.parquet.output.committer.class": "org.apache.spark.internal.io.cloud.BindingParquetOutputCommitter",
        "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
        "spark.sql.catalog.spark_catalog": "org.apache.spark.sql.delta.catalog.DeltaCatalog"
    }


    def get_spark_session(app_name: str, conf: SparkConf):
        spark = SparkSession.builder.master("local[*]").appName(app_name)

        for key, value in config.items():
            spark.config(key, value)
        return spark.getOrCreate()




    spark = get_spark_session(app_name="test", conf=config)

    sc = spark.sparkContext
    print(f"Hadoop version = {sc._jvm.org.apache.hadoop.util.VersionInfo.getVersion()}")

    return spark


def test_write(spark, df):

    df.write.format("delta").mode("append").save(PATH)


def test_read(spark):

    df = spark.read.format("delta").load(PATH)

    df.printSchema()
    print(df.count())

    df.show(10)





from delta import DeltaTable
from pyspark.sql import SparkSession

def get_delta_table_metadata(spark):

    # Load the Delta table as a DeltaTable object
    delta_table = DeltaTable.forPath(spark, PATH)

    # Get the metadata for the Delta table
    history = delta_table.history()
    history.show(truncate=False)



    # Stop the SparkSession object
    # spark.stop()


import datetime
import random
from pyspark.sql import SparkSession
from pyspark.sql.functions import col

def generate_fake_data(num_rows: int) -> SparkSession:
    spark = SparkSession.builder.appName("FakeData").getOrCreate()
    # Define the schema for the DataFrame
    schema = "event_time TIMESTAMP, key INT, value STRING"
    # Generate the fake data
    rows = []
    for i in range(num_rows):
        timestamp = datetime.datetime.now() - datetime.timedelta(days=random.randint(1, 365))
        key = random.randint(1, 10)
        value = f"value_{i}"
        rows.append((timestamp, key, value))
    # Create the DataFrame
    df = spark.createDataFrame(rows, schema)
    # Partition the DataFrame by the "event_time" column



    df = df.repartition(col("event_time"), col("key"))
    return df


if __name__ == "__main__":

    spark = spark_config()

    df = generate_fake_data(1000)

    test_write(spark, df)

    get_delta_table_metadata(spark)

    test_read(spark)
