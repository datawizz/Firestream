from pyspark.sql import SparkSession
from pyspark.sql.functions import to_json, struct
from etl_lib.services.spark.client import SparkClient
from threading import Thread
from kafka import KafkaConsumer
import json

BOOTSTRAP_SERVERS = "kafka-0.kafka-headless.default.svc.cluster.local:9092"  # Required for spark to find leader election?
TOPIC = "a_test_topic"
MAX_MESSAGES = 100000


class KafkaMessageMonitor(Thread):
    def __init__(self, bootstrap_servers, topic):
        super().__init__()
        self.consumer = KafkaConsumer(topic, bootstrap_servers=[bootstrap_servers])
        self.counter = 0

    def run(self):
        for message in self.consumer:
            self.counter += 1
            if self.counter >= MAX_MESSAGES:
                break

    def get_counter(self):
        return self.counter


def test_spark_to_kafka():
    spark = SparkClient("StructuredNetworkWordCount")
    spark = spark.session

    df = (
        spark.readStream.format("rate-micro-batch")
        .option("rowsPerBatch", 100000)
        .option("numPartitions", 3)
        .option("advanceMillisPerBatch", 1000)
        .load()
    )

    # Format the payload for Kafka
    df = df.withColumn(
        "value",
        to_json(struct("*")).cast("string"),
    ).select("value")

    kafka_monitor = KafkaMessageMonitor(BOOTSTRAP_SERVERS, TOPIC)
    kafka_monitor.start()

    query = df.writeStream.outputMode("append").format("kafka").option("topic", TOPIC).option(
        "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
    ).start()

    while True:
        if kafka_monitor.get_counter() >= MAX_MESSAGES:
            query.stop()
            break

    kafka_monitor.join()
    query.awaitTermination()


if __name__ == "__main__":
    test_spark_to_kafka()
