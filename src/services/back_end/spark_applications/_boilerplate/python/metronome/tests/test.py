# Smoketest for Kafka connectivity


"""
** Please be patient while the chart is being deployed **

Kafka can be accessed by consumers via port 9092 on the following DNS name from within your cluster:

    kafka.default.svc.cluster.local

Each Kafka broker can be accessed by producers via port 9092 on the following DNS name(s) from within your cluster:

    kafka-0.kafka-headless.default.svc.cluster.local:9092
    kafka-1.kafka-headless.default.svc.cluster.local:9092
    kafka-2.kafka-headless.default.svc.cluster.local:9092

"""

# # TODO pull this from environment variables
# BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"
# BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"

BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"


from confluent_kafka import Producer, Consumer
from confluent_kafka.admin import AdminClient, NewTopic, ClusterMetadata, TopicMetadata

from datetime import datetime
from uuid import uuid4
import json
import uuid



def test_consume(topic: str):
    """
    Read data from the provided topic
    """

    c = Consumer(
        {
            "bootstrap.servers": BOOTSTRAP_SERVERS,
            "group.id": uuid.uuid4().hex,
            "auto.offset.reset": "latest",
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


    test_consume("metronome")



