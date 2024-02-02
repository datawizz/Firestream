# Smoketest for Kafka connectivity
# Attempts to retrieve events from Kafka

from confluent_kafka import Producer, Consumer
from confluent_kafka.admin import AdminClient, NewTopic, ClusterMetadata, TopicMetadata

from datetime import datetime
from uuid import uuid4
import json

BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"

def test_consume(topic: str):
    """
    Read data from the provided topic
    """

    c = Consumer(
        {
            "bootstrap.servers": BOOTSTRAP_SERVERS,
            "group.id": "mygroup",
            "auto.offset.reset": "earliest",
        }
    )

    c.subscribe([topic])

    while True:
        msg = c.poll(1.0)

        if msg is None:
            continue
        if msg.error():
            print("Consumer error: {}".format(msg.error()))
            continue

        print("Received message: {}".format(msg.value().decode("utf-8")))

    c.close()



if __name__ == "__main__":

    test_consume("spark_stateful")


