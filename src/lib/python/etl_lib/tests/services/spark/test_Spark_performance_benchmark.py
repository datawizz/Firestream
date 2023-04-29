from pyspark.sql import SparkSession
from pyspark.sql.functions import explode
from pyspark.sql.functions import split


def vanilla():
    spark = SparkSession.builder.appName("StructuredNetworkWordCount").getOrCreate()

    query = (
        spark.readStream.format("rate-micro-batch")
        .option("rowsPerBatch", 20)
        .option("numPartitions", 3)
        .load()
        .writeStream.format("console")
        .start()
    )

    query.awaitTermination()


from etl_lib.services.spark.client import SparkClient

from pyspark.sql import SparkSession
import pyspark.sql.functions as F

BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"
BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
BOOTSTRAP_SERVERS = "kafka-0.kafka-headless.default.svc.cluster.local:9092"  # Required for spark to find leader election?
TOPIC = "a_test_topic"
from datetime import datetime
import pandas as pd


def test_spark_to_kafka():

    spark = SparkClient("StructuredNetworkWordCount")
    spark = spark.spark_session

    df = (
        spark.readStream.format("rate-micro-batch")
        .option("rowsPerBatch", 100000)
        .option("numPartitions", 3)
        .option("advanceMillisPerBatch", 1000)
        # .option("startTimestamp", pd.to_datetime("2022-01-01"))
        .load()
    )
    # df.show()

    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct("*")).cast("string"),
    ).select("value")

    # df.writeStream.format("console").start().awaitTermination()

    df.writeStream.outputMode("append").format("kafka").option("topic", TOPIC).option(
        "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
    ).start().awaitTermination()


if __name__ == "__main__":
    test_spark_to_kafka()
    # vanilla()
