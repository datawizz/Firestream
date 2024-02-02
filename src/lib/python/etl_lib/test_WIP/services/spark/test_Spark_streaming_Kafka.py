from pyspark.sql import SparkSession
from pyspark.sql.functions import *
from pyspark.sql.types import *

KAFKA_TOPIC_NAME = "mytopic"
KAFKA_SINK_TOPIC = "another_topic"
BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"
BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"

CHECKPOINT_LOCATION = "/workspace/.data/spark/checkpoint"
# CHECKPOINT_LOCATION = "/Users/aman.parmar/Documents/DATA_SCIENCE/KAFKA/CHECKPOINT"


from etl_lib.services.spark.client import SparkClient

if __name__ == "__main__":

    # STEP 1 : creating spark session object

    sparkApp = SparkClient("test_streaming_spark_smoketest")
    spark = sparkApp.spark_context

    # STEP 2 : reading a data stream from a kafka topic

    sampleDataframe = (
        spark.readStream.format("kafka")
        .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
        .option("subscribe", KAFKA_TOPIC_NAME)
        .option("startingOffsets", "latest")
        .load()
    )

    base_df = sampleDataframe.selectExpr("CAST(value as STRING)", "timestamp")
    base_df.printSchema()

    # STEP 3 : Applying suitable schema

    sample_schema = (
        StructType()
        .add("col_a", StringType())
        .add("col_b", StringType())
        .add("col_c", StringType())
        .add("col_d", StringType())
    )

    info_dataframe = base_df.select(
        from_json(col("value"), sample_schema).alias("info"), "timestamp"
    )

    info_dataframe.printSchema()
    info_df_fin = info_dataframe.select("info.*", "timestamp")
    info_df_fin.printSchema()

    # STEP 4 : Creating query using structured streaming

    query = info_df_fin.groupBy("col_a").agg(
        approx_count_distinct("col_b").alias("col_b_alias"),
        count(col("col_c")).alias("col_c_alias"),
    )

    # query = query.withColumn("query", lit("QUERY3"))
    result_1 = query.selectExpr(
        "CAST(col_a AS STRING)",
        "CAST(col_b_alias AS STRING)",
        "CAST(col_c_alias AS STRING)",
    ).withColumn(
        "value",
        to_json(struct("*")).cast("string"),
    )

    result = (
        result_1.select("value")
        .writeStream.trigger(processingTime="10 seconds")
        .outputMode("complete")
        .format("kafka")
        .option("topic", KAFKA_SINK_TOPIC)
        .option("kafka.bootstrap.servers", BOOTSTRAP_SERVERS)
        .option("checkpointLocation", CHECKPOINT_LOCATION)
        .start()
        .awaitTermination()
    )

    # result.awaitTermination()
