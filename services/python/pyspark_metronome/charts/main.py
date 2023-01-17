

import json, uuid, os, time
from datetime import datetime
from pyspark.sql import SparkSession, DataFrame
import pyspark.sql.functions as F
from pyspark.sql.types import StructType, StructField, StringType, ArrayType


BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
DESTINATION_TOPIC = "metronome"
SPARK_MASTER = "spark://spark-master-svc.default.svc.cluster.local:7077"
SPARK_APP_NAME = "SparkMetronome"
DEBUG = False
DEVICES = 5

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


def create_signals() -> DataFrame:

    spark = spark_context()

    now = datetime.now()
    unix_time_millis = int(time.mktime(now.timetuple())*1000)

    df = (
        spark.readStream.format("rate-micro-batch")
        .option("rowsPerBatch", 1)
        .option("numPartitions", 1)
        .option("advanceMillisPerBatch", 1000)
        .option("startTimestamp", unix_time_millis)
        .load()
    )

    return df


def log_signals(df: DataFrame) -> None:

    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct("*")).cast("string"),
    ).select("value")

    df = df.withColumn("key", F.sha1(F.col("value")))

    if DEBUG:
        # Write the stream to the conole
        df.writeStream.outputMode("append").format("console").option("truncate", "false").start().awaitTermination()

    else:
        # Write the stream to Kafka
        df.writeStream.outputMode("append").format("kafka").option("topic", DESTINATION_TOPIC).option(
            "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
        ).start().awaitTermination()




def pipeline(df: DataFrame) -> DataFrame:

    df = df.withColumn('timestamp', df.timestamp.cast('timestamp')).withColumnRenamed("timestamp", "event_time")

    # Generate random device ids and broadcast this small piece of state
    device_ids = json.dumps([uuid.uuid4().hex for x in range(DEVICES)])
    spark = spark_context()
    spark.sparkContext.broadcast(device_ids)

    # Create a column with the device ids and then explode it into rows
    df = df.withColumn("device_id", F.lit(device_ids))
    df = df.withColumn("device_id", F.from_json(F.col("device_id"), schema=ArrayType(StringType())))
    df = df.withColumn("device_id", F.explode(F.col("device_id")))

    # Sampling from a normal distribution between 0 and 1 for magnitude 
    df = df.withColumn("magnitude", F.rand(42))

    # Sampling from a normal distribution between -1 and 1 for direction
    # round to 0 for down and 1 for up
    df = df.withColumn("direction", F.round(F.rand(42),0))

    # value = index

    # Update the key to be the Device ID
    df = df.withColumn("key", F.col("device_id"))

    # Update the "value" to be the index
    df = df.withColumnRenamed("value","index")

    return df


if __name__ == "__main__":
    
    df = create_signals()
    df = pipeline(df)
    log_signals(df)