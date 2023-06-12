from pyspark.sql import SparkSession
from pyspark.sql.functions import from_json, col
from pyspark.sql.types import StructType, StructField, StringType, DoubleType, TimestampType, IntegerType
import os
import pyspark.sql.functions as F
import json, uuid, os, time
from datetime import datetime
from pyspark.sql import SparkSession, DataFrame
import pyspark.sql.functions as F
from pyspark.sql.types import StructType, StructField, StringType, ArrayType


DESTINATION_TOPIC = "metronome_processed"
BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
SOURCE_TOPIC = "metronome"
SPARK_MASTER = "local[4]"
SPARK_APP_NAME = "SparkMetronomeReceiveContinuous"
DEBUG = True


# Define the schema of the incoming data
schema = StructType([
    StructField("event_time", TimestampType(), True),
    StructField("index", IntegerType(), True),
    StructField("device_id", StringType(), True),
    StructField("magnitude", DoubleType(), True),
    StructField("direction", DoubleType(), True),
    StructField("key", StringType(), True)
])



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
        "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint",
    }

    spark = SparkSession.builder.master(SPARK_MASTER).appName(SPARK_APP_NAME)

    for key, value in config.items():
        spark.config(key, value)
        
    return spark.getOrCreate()




def log_signals(df: DataFrame) -> None:

    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct([col for col in df.columns if col != "key"])).cast("string"),
    )
    # df = df.select("value", "key")

    # df = df.withColumn("key", F.sha1(F.col("value")))

    if DEBUG:
        # Write the stream to the conole
        df.writeStream.outputMode("append").format("console").option("truncate", "false").start().awaitTermination()

    else:
        # Write the stream to Kafka
        df.writeStream.outputMode("append").format("kafka").option("topic", DESTINATION_TOPIC).option(
            "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
        ).start().awaitTermination()


def _count(df):



    # Cast the 'event_time' column to a timestamp
    df = df.withColumn("event_time", df["event_time"].cast("timestamp")).withWatermark('event_time', '1 seconds')

    # Group by window of 5 seconds
    df = df.groupBy(
        F.window(df.event_time, "5 seconds")
    ).agg(
        F.count("index").alias("count"),
        F.mean("magnitude").alias("avg_magnitude"),
        F.mean("direction").alias("avg_direction"),
    )

    return df

if __name__ == "__main__":
    # Create a SparkSession
    spark = spark_context()

    # Read data from Kafka
    kafka_df = spark \
        .readStream \
        .format("kafka") \
        .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS) \
        .option("subscribe", SOURCE_TOPIC) \
        .option("startingOffsets", "latest") \
        .load()

    # Convert the binary key and value columns to string
    kafka_df = kafka_df.selectExpr("CAST(key AS STRING)", "CAST(value AS STRING)")
    
    # Convert the JSON string in the value column to structured data
    json_df = kafka_df.select(from_json(col("value"), schema).alias("data"))

    # Select the columns from the structured data
    data_df = json_df.select("data.*")

    data_df = data_df.withColumn("key", F.col("device_id"))

    data_df = _count(data_df)

    # Write the data to Kafka
    log_signals(data_df)
