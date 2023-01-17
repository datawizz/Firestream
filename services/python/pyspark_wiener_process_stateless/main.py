

import json, uuid, os, time
from datetime import datetime
from pyspark.sql import SparkSession, DataFrame
import pyspark.sql.functions as F
from pyspark.sql.types import *
# from pyspark.sql.types import StructType, StructField, StringType, ArrayType




BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
SOURCE_TOPIC = "metronome"
DESTINATION_TOPIC = "pyspark_wiener_process_stateless"
SPARK_MASTER = "spark://spark-master-svc.default.svc.cluster.local:7077"
SPARK_MASTER = "local[*]"
SPARK_APP_NAME = "PySpark_Wiener_Process_Stateless"
DEBUG = False

def spark_context() -> None:

    jars_packages = [
        # For Kafka access
        "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
        "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
        "org.apache.kafka:kafka-clients:3.3.1",
    ]

    repos = [
        "https://packages.confluent.io/maven",
    ]

    jars = ",".join(jars_packages)
    repos = ",".join(repos)

    args = f"--packages {jars} --repositories {repos} pyspark-shell"
    os.environ["PYSPARK_SUBMIT_ARGS"] = args

    config = {
        "spark.sql.execution.arrow.pyspark.enabled": "true",
        "spark.sql.session.timeZone": "UTC",
        "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint"
    }

    spark = SparkSession.builder.master(SPARK_MASTER).appName(SPARK_APP_NAME)

    for key, value in config.items():
        spark.config(key, value)
    spark = spark.getOrCreate()
    return spark


def read_stream() -> DataFrame:

    spark = spark_context()

    df = spark \
        .readStream \
        .format("kafka") \
        .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS) \
        .option("subscribe", SOURCE_TOPIC) \
        .option("enable.auto.commit", "false") \
        .option("kafkaConsumer.pollTimeoutMs", "10") \
        .load()


    # Define the schema for the input data
    input_schema = StructType([
        StructField("event_time", TimestampType()),
        StructField("index", IntegerType()),
        StructField("device_id", StringType()),
        StructField("magnitude", DoubleType()),
        StructField("direction", DoubleType()),
        StructField("key", StringType())
    ])

    # Parse the JSON data and select the relevant columns
    df = df.selectExpr("cast(value as STRING) as json_data", "cast(timestamp as TIMESTAMP) as kafka_write_timestamp") \
        .select(F.from_json("json_data", input_schema).alias("data"), "kafka_write_timestamp") \
        .select("data.*", "kafka_write_timestamp")

    return df


def write_stream(df: DataFrame) -> None:

    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct([col for col in df.columns if col != "key"])).cast("string"),
    )
    df = df.select("value", "key")

    # df = df.withColumn("key", F.sha1(F.col("value")))

    if DEBUG:
        # Write the stream to the conole
        df.writeStream.outputMode("append").format("console").option("truncate", "false").trigger(continuous = "5 seconds").start().awaitTermination()

    else:
        # Write the stream to Kafka
        df.writeStream.outputMode("append").format("kafka").option("topic", DESTINATION_TOPIC).option(
            "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
        ).trigger(continuous = "5 seconds").start().awaitTermination()


def pipeline(df):
    df = df.withColumn("direction", F.when(F.col("direction") == 0, -1).otherwise(1))
    df = df.withColumn("magnitude", F.col("magnitude") * F.col("direction"))
    df = df.withColumnRenamed("magnitude", "value")
    df = df.select("index", "value", "key", "device_id", "event_time")

    #TODO include logic to evaluate the delay between the event_time and the "kafka_write_timestamp"
    # boil this up to the front.
    #|{"event_time":"2023-01-18T22:32:03.000Z","index":186776,"device_id":"c082613352cf48d695f5d376a9730310","magnitude":0.677695494840249,"direction":1,"kafka_write_timestamp":"2023-01-17T01:32:38.481Z"}    |dd2b7ad5b0139759c2ee24e72a1a51b3d1cd728a|
    
    return df



if __name__ == "__main__":
    
    df = read_stream()
    df = pipeline(df)
    write_stream(df)