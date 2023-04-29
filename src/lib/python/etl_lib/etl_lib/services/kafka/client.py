import logging

from confluent_kafka import SerializingProducer
from confluent_kafka import Consumer, DeserializingConsumer
from confluent_kafka.serialization import StringSerializer
from confluent_kafka.schema_registry import SchemaRegistryClient
from confluent_kafka.schema_registry.avro import AvroSerializer
from confluent_kafka.avro.serializer import SerializerError
from confluent_kafka.admin import AdminClient, NewTopic
from confluent_kafka.schema_registry.schema_registry_client import SchemaRegistryClient
from confluent_kafka.schema_registry import Schema
from fastavro import schema


# TODO pull this from environment variables
BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"


class KafkaClient:

    """
    Wraps the Confluent Kafka python library (which wraps the C library librdkafka for speed)
    Provides admin interface to create and config topics.
    Provides consistent configs to standardize access accross the project
    """

    def __init__(self):

        """
        Init static configurations
        """
        # Set Resource Locations
        # TODO make this a DNS name in Kubernetes rather than a IP address
        self.bootstrap_servers_dict = {"bootstrap.servers": BOOTSTRAP_SERVERS}
        self.schema_registry_dict = {"url": "http://127.0.0.1:8081"}
        self.config = (
            self.bootstrap_servers_dict
        )  # TODO make this cover the schema registry too
        self.schema_registry_client = None
        self.serializer = None
        self.producer = None
        self.produced = 0
        self.idempotence = 1
        self.consumer_client = None
        # self.admin_client = AdminClient(self.bootstrap_servers_dict)

        # self.logger = self.make_logger()

    @staticmethod
    def delivery_report(err, msg):
        """Called once for each message produced to indicate delivery result.
        Triggered by poll() or flush()."""
        if err is not None:
            print("Message delivery failed: {}".format(err))
        else:
            return
            print("Message delivered to {} [{}]".format(msg.topic(), msg.partition()))

    @staticmethod
    def throttle_report():
        print("throttling")

    def produce(self, payload: dict, schema_str: str, topic: str, flush=False):

        """
        Produces an instance of a DataModel to it's Kafka Topic
        """

        self.produced = self.produced + 1

        if not self.schema_registry_client:
            self.schema_registry_client = SchemaRegistryClient(
                self.schema_registry_dict
            )

        # TODO #3 detect multiple model inputs?
        if not self.producer:

            self.serializer = AvroSerializer(
                schema_str=schema_str,
                schema_registry_client=self.schema_registry_client,
            )

            producer_config = {
                "bootstrap.servers": self.bootstrap_servers_dict.get(
                    "bootstrap.servers"
                ),
                "key.serializer": StringSerializer("utf_8"),
                "value.serializer": self.serializer,
                "enable.idempotence": self.idempotence,
                "throttle_cb": self.throttle_report,
                # "log_cb": self.logger,
            }
            self.producer = SerializingProducer(producer_config)
        # # Every 10000 events ensure cache is cleared
        if self.produced % 10000 == 0:
            print(self.produced)
            self.producer.poll(0)

        self.producer.produce(
            topic=topic,
            key=payload["key"],
            value=payload["value"],
            on_delivery=self.delivery_report,
        )

        # makes it synchronous for testing: https://stackoverflow.com/a/52591213
        if flush:
            self.producer.flush()

    def create_or_update_topic(self, topic: str, schema_str: str):
        """

        Given a Avro Schema and a Topic in Kafka register the schema with the SchemaRegistry
        If the schema already exists in the registry attempt to upgrade it.
        Return an error if it cannot be eveolved.

        DataFactory create a matching topic in Kafka and register the schema.
        If the provided schema is different than an existing schema of the same topic name,
            attempt to evolve it or produce an error.

        # Ensure the "required/non-nullable" flags are consistent
        # Ensure the types are consistent for each field


        # Create some sort of meta-data topic which includes easy to use references for each topic.
        Maybe make this in KSQL?
            Min offset 0
            Max offset 123...
            Min event timestamp
            Max event timestamp
            Count of records




        Use Tinsel, JSON wizard, AvroSchema

        Use the Admin API of Kafka
        """

        print(schema_str)
        print(topic)

        topic_list = [NewTopic(topic, num_partitions=3, replication_factor=1)]

        self.admin_client.create_topics(new_topics=topic_list, validate_only=False)

        sr = SchemaRegistryClient(
            {
                "url": "http://localhost:8081",
                # "ssl.certificate.location": "/path/to/cert",  # optional
                # "ssl.key.location": "/path/to/key",  # optional
            }
        )

        sr.register_schema(
            subject_name=topic, schema=Schema(schema_str=schema_str, schema_type="AVRO")
        )

        # value_schema = sr.get_latest_schema(f"{topic}-value")[1]
        # key_schema = sr.get_latest_schema(f"{topic}-key")[1]

        # print(value_schema, key_schema)

    def get_topic(self, topic: str):
        """
        Collect meta data about the topic self
        """
        # TODO make this more elegant by including in init?
        if not self.schema_registry_client:
            self.schema_registry_client = SchemaRegistryClient(
                self.schema_registry_dict
            )

        # confluent_kafka.admin.TopicMetadata
        name = self.schema_registry_client.get_subjects()
        print(name)  # What the fuck is a subject?

        # key_schemna = self.schema_registry_client.get_latest_version(f"{topic}-key")
        # value_schema = self.schema_registry_client.get_latest_version(f"{topic}-value")
        # schema = self.schema_registry_client.get_compatibility(subject_name=name[1])

    def consumer(self):

        groupd_id = "the_name_of_the_model"

        self.config.update({"group.id": groupd_id})

        self.consumer_client = Consumer(self.config)

        return self.consumer_client

    def get_topics(self, topic: str):
        """
        Retrieve Meta Data about the Topic
        """
        if not self.consumer_client:
            self.consumer_client = self.consumer()

        data = self.consumer_client.list_topics(topic=topic)
        print(data)

    def get_metadata(self, topic: str):
        """ """

        admin_client = AdminClient(self.bootstrap_servers_dict)

        cluster_metadata = admin_client.list_topics(timeout=10, topic=topic)
        # print(cluster_metadata.topics)
        # print(cluster_metadata.brokers)
        return cluster_metadata.topics, cluster_metadata.brokers
