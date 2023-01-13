import os
from pyspark.sql import SparkSession


import pyspark.sql.functions as F


BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
DESTINATION_TOPIC = "spark_metronome1"
SPARK_MASTER = "spark://spark-master-svc.default.svc.cluster.local:7077" #local[*]

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

    spark = spark_context("SparkMetronome")

    df = (
        spark.readStream.format("rate-micro-batch")
        .option("rowsPerBatch", 1)
        .option("numPartitions", 1)
        .option("advanceMillisPerBatch", 1000)
        #.option("startTimestamp", pd.to_datetime("2022-01-01"))
        .load()
    )


    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        F.to_json(F.struct("*")).cast("string"),
    ).select("value")

    df = df.withColumn("key", F.sha1(F.col("value")))
    #df.writeStream.outputMode("append").format("console").start().awaitTermination()
    # Write the stream to Kafka
    df.writeStream.outputMode("append").format("kafka").option("topic", DESTINATION_TOPIC).option(
        "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
    ).start().awaitTermination()


if __name__ == "__main__":
    pipeline()
