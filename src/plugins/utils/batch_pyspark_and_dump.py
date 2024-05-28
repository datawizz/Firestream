from pyspark.sql import SparkSession
from etl_lib import SparkClient
import os

_TMP_DIR = '/tmp/spark'
os.makedirs(_TMP_DIR, exist_ok=True)

class KafkaParquetHandler:
    def __init__(self, broker_address, topic_name):
        self.broker_address = broker_address
        self.topic_name = topic_name
        self.spark = SparkClient(app_name="PySpark and Dump2").spark_session

    def _append_to_parquet(self, df, file_name):
        df.write.option("compression", "snappy").mode('append').parquet(file_name)

    def read_from_topic_write_to_parquet(self, file_name):
        df = self.spark.read \
            .format("kafka") \
            .option("kafka.bootstrap.servers", self.broker_address) \
            .option("subscribe", self.topic_name) \
            .option("startingOffsets", "earliest") \
            .option("fetch.message.max.bytes", "1048576") \
            .load()

        df_transformed = df.selectExpr(
            "timestamp",
            "CAST(key AS STRING)",
            "CAST(value AS STRING)"
        )

        self._append_to_parquet(df_transformed, file_name)

    def read_from_parquet_write_to_topic(self, file_name):
        df = self.spark.read.parquet(file_name)
        df.write \
            .format("kafka") \
            .option("kafka.bootstrap.servers", self.broker_address) \
            .option("topic", self.topic_name) \
            .save()

if __name__ == "__main__":
    handler = KafkaParquetHandler(broker_address=os.environ['KAFKA_BOOTSTRAP_SERVERS'], topic_name='A_RUSTY_TOPIC')
    handler.read_from_topic_write_to_parquet('/workspace/src/lib/python/etl_lib/utils/output_file_pyspark.parquet')
