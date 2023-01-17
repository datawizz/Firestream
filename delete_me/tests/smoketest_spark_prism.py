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


def test_produce(topic: str):
    """
    Write simple random data to Kafka
    """
    p = Producer({"bootstrap.servers": BOOTSTRAP_SERVERS})

    def some_data_source():

        for i in range(100):
            payload = {
                "key": str(datetime.now().timestamp() * 10**6),
                "value": {
                    "name": str(uuid4()),
                    "favorite_number": str(uuid4()),
                    "favorite_color": str(uuid4()),
                },
            }
            yield json.dumps(payload)

    def delivery_report(err, msg):
        """Called once for each message produced to indicate delivery result.
        Triggered by poll() or flush()."""
        if err is not None:
            print("Message delivery failed: {}".format(err))
        else:
            print("Message delivered to {} [{}]".format(msg.topic(), msg.partition()))

    for data in some_data_source():
        # Trigger any available delivery report callbacks from previous produce() calls
        p.poll(0)

        # Asynchronously produce a message. The delivery report callback will
        # be triggered from the call to poll() above, or flush() below, when the
        # message has been successfully delivered or failed permanently.
        p.produce(topic, data.encode("utf-8"), callback=delivery_report)

    # Wait for any outstanding messages to be delivered and delivery report
    # callbacks to be triggered.
    p.flush()


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


def test_create_topic(topic: str):

    a = AdminClient({"bootstrap.servers": BOOTSTRAP_SERVERS})
    print(a.list_topics().__dict__)

    new_topics = [NewTopic(topic, num_partitions=3, replication_factor=1)]
    # Note: In a multi-cluster production scenario, it is more typical to use a replication_factor of 3 for durability.

    # Call create_topics to asynchronously create topics. A dict
    # of <topic,future> is returned.
    fs = a.create_topics(new_topics)

    # Wait for each operation to finish.
    for topic, f in fs.items():
        try:
            f.result()  # The result itself is None
            print("Topic {} created".format(topic))
        except Exception as e:
            print("Failed to create topic {}: {}".format(topic, e))


# def test_topic_metadata(topic:str):
#     """
#     List the meta data for the topic
#     """

#         a = AdminClient({"bootstrap.servers": BOOTSTRAP_SERVERS})


if __name__ == "__main__":

    #test_produce("a_test_topic")
    test_consume("spark_prism")
    # test_create_topic("mytopic")


