from pyspark.sql import SparkSession
from pyspark.sql.functions import from_json, col, to_timestamp, expr
from pyspark.sql.types import StructType, StructField, StringType, DoubleType, TimestampType, IntegerType, LongType
import os

from pyspark import SparkConf
SOURCE_TOPIC = "metronome"
BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
DESTINATION_TOPIC = "metronome"
BUCKET = "test"


os.environ["AWS_ACCESS_KEY_ID"] = "minio"
os.environ["AWS_SECRET_ACCESS_KEY"] = "miniominio"
os.environ["AWS_ENDPOINT_URL"] = "http://10.0.0.67:9000"
os.environ["AWS_BUCKET"] = "test"

def spark_config():

    jars_packages = [
        # For Kafka access
        "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
        "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
        "org.apache.kafka:kafka-clients:3.3.1",
        "org.apache.hadoop:hadoop-aws:3.3.1",
        "org.apache.hadoop:hadoop-common:3.3.1",
        "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1",
        "io.delta:delta-core_2.12:2.2.0"
    ]

    repos = [
        "https://packages.confluent.io/maven",
    ]

    jars = ",".join(jars_packages)

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
        "spark.hadoop.fs.s3a.endpoint": os.environ.get("AWS_ENDPOINT_URL"),
        "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
        "spark.hadoop.fs.s3a.path.style.access": "true",
        "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
        "spark.hadoop.com.amazonaws.services.s3.enableV4": "true",
        # "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.AnonymousAWSCredentialsProvider",
        "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
        "spark.hadoop.fs.s3a.access.key": os.environ.get("AWS_ACCESS_KEY_ID"),
        "spark.hadoop.fs.s3a.secret.key": os.environ.get("AWS_SECRET_ACCESS_KEY"),
        "spark.jars.packages": jars,
        # "spark.hadoop.parquet.summary.metadata.level": "false",
        "spark.sql.parquet.mergeSchema": "false",
        "spark.sql.parquet.filterPushdown": "true",
        "spark.sql.hive.metastorePartitionPruning": "true",
        "spark.hadoop.fs.s3a.committer.name": "directory",
        "spark.sql.sources.commitProtocolClass": "org.apache.spark.internal.io.cloud.PathOutputCommitProtocol",
        "spark.sql.parquet.output.committer.class": "org.apache.spark.internal.io.cloud.BindingParquetOutputCommitter",
        "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
        "spark.sql.catalog.spark_catalog": "org.apache.spark.sql.delta.catalog.DeltaCatalog",
        "spark.hadoop.fs.s3a.multipart.size": "104857600"
        # "spark.eventLog.enabled": "true",
        # "spark.eventLog.dir": F"s3a://{BUCKET}/logs",
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


# def spark_context() -> None:

#     jars_packages = [
#         # For Kafka access
#         "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
#         "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
#         "org.apache.kafka:kafka-clients:3.3.1",
#         "org.apache.hadoop:hadoop-aws:3.3.1",
#         "org.apache.hadoop:hadoop-common:3.3.1",
#         "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1",
#         "io.delta:delta-core_2.12:2.2.0"
#     ]

#     repos = [
#         "https://packages.confluent.io/maven",
#     ]

#     jars = ",".join(jars_packages)
#     repos = ",".join(repos)

#     args = f"--packages {jars} --repositories {repos} pyspark-shell"
#     os.environ["PYSPARK_SUBMIT_ARGS"] = args

#     config = {
#         "spark.sql.execution.arrow.pyspark.enabled": "true",
#         "spark.sql.session.timeZone": "UTC",
#         "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint",
#     }

#     spark = SparkSession.builder.master("local[*]").appName("KafkaSparkStreaming")

#     for key, value in config.items():
#         spark.config(key, value)
#     return spark.getOrCreate()



# Create SparkSession
spark = spark_config()

# Define schema for Kafka topic
# Define schema for the streaming data
kafka_schema = StructType([
        StructField("event_time", StringType()),
        StructField("index", LongType()),
        StructField("device_id", StringType()),
        StructField("magnitude", DoubleType()),
        StructField("direction", DoubleType())
])

# Define options for reading from Kafka
kafka_options = {
    "kafka.bootstrap.servers": BOOTSTRAP_SERVERS,
    "subscribe": SOURCE_TOPIC,
    "startingOffsets": "latest"
}

# Define S3 output path and Delta table name
s3_output_path = F"s3a://{BUCKET}/my-path4"
delta_table_name = "my_delta_table3"

# Define write options for writing to Delta table
delta_write_options = {
    "path": s3_output_path,
    "checkpointLocation": f"s3a://{BUCKET}/my-path4/checkpoint",
    "mergeSchema": "true" ###!!!!
}

# Read from Kafka and parse JSON data
kafka_df = spark \
    .readStream \
    .format("kafka") \
    .options(**kafka_options) \
    .load() \
    # .selectExpr("CAST(value AS STRING)") \
    # .select(from_json(col("value"), kafka_schema).alias("data")) \
    # .select("data.key", "data.timestamp", "data.partition", "data.value.*")

# # Convert timestamp to timestamp type
# kafka_df = kafka_df.withColumn("event_time", to_timestamp(col("timestamp")))

# Write to Delta table in S3
kafka_df \
    .writeStream \
    .format("delta") \
    .outputMode("append") \
    .options(**delta_write_options) \
    .option("checkpointLocation", delta_write_options["checkpointLocation"] + "/" + delta_table_name) \
    .queryName(delta_table_name) \
    .start()

# Start the streaming query
spark.streams.awaitAnyTermination()
