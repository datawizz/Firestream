from pyspark import SparkConf
from pyspark.sql import SparkSession
import os


from etl_lib.services.spark.client import SparkClient
from etl_lib import DataModel, DataFactory
from etl_lib.services.kafka.client import KafkaClient


def test_spark_client_s3_read_write():

    from etl_lib.services.spark.client import SparkClient

    spark = SparkClient(app_name="a_test").spark_session

    sc = spark.sparkContext
    spark.sparkContext.setLogLevel("WARN")
    # print(f"Hadoop version = {sc._jvm.org.apache.hadoop.util.VersionInfo.getVersion()}")

    df = spark.read.csv("../example/data/cities.csv")

    df.printSchema()
    print(df.count())

    df.write.mode("overwrite").format("parquet").save(
        "s3a://data/example/data/cities_parquet"
    )


    df = spark.read.csv("s3a://data/example/data/cities.csv")
    # df = spark.read.csv("file:///workspace/.data/cities.csv")
    df.printSchema()
    print(df.count())

    spark.stop()

if __name__ == "__main__":
    test_spark_client_s3_read_write()
