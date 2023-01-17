import os
from pyspark.sql import SparkSession
import pyspark.sql.functions as F
from pyspark.sql.types import StructType,StructField, StringType

from pyspark.sql.window import Window
from pyspark.sql.functions import row_number

import json


from typing import Iterator
import pandas as pd
import numpy as np
from pyspark.sql.functions import pandas_udf

import uuid

from pyspark.sql.types import StringType, ArrayType


BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
SPARK_MASTER = "spark://spark-master-svc.default.svc.cluster.local:7077"
SPARK_MASTER = "local[1]"
SOURCE_TOPIC = "spark_metronome1"
DESTINATION_TOPIC = "spark_prism"
DEVICES = 5

# example_data = [
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381937},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390228},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391379},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381940},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390231},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391382},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381943},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390234},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391385},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381946},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390237},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391388},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381949},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390240},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391391},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381952},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390243},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391394},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461381955},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461390246},
#     {"timestamp": "1970-01-01T01:16:53.000Z", "value": 461391397},
# ]


def spark_context(app_name: str) -> None:

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

    spark = SparkSession.builder.master(SPARK_MASTER).appName(app_name)

    for key, value in config.items():
        spark.config(key, value)
    return spark.getOrCreate()


def pipeline():

    spark = spark_context("SparkPrism")

    df = (
        spark.readStream.format("kafka")
        .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
        .option("subscribe", SOURCE_TOPIC)
        .option("startingOffsets", "latest")
        .option("includeHeaders", "true")
        .load()
    )
    df = df.selectExpr("partition","offset","timestamp","timestampType","headers","CAST(key AS STRING)", "CAST(value AS STRING)")


    # Create Schema of the JSON column
    schema = StructType([ 
        StructField("value",StringType(),True)
    ])
    df = df.withColumn("jsonData",F.from_json(F.col("value"),schema)).select("timestamp", "key", "jsonData.*")

    df = df.withColumn('timestamp', df.timestamp.cast('timestamp'))






    # Generate random device ids and broadcast this small piece of state
    device_ids = json.dumps([uuid.uuid4().hex for x in range(DEVICES)])
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




    # df.writeStream.outputMode("append").format("console").start().awaitTermination()


    # # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct("*")).cast("string"),
    ).select("key","value")

    df.writeStream.outputMode("append").format("kafka").option(
        "topic", DESTINATION_TOPIC
    ).option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS).start().awaitTermination()


if __name__ == "__main__":
    pipeline()
