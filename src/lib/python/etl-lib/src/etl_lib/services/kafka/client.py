import os
from typing import Optional, Tuple

from confluent_kafka import Consumer, SerializingProducer
from confluent_kafka.admin import AdminClient, NewTopic
from confluent_kafka.schema_registry import Schema, SchemaRegistryClient
from confluent_kafka.schema_registry.avro import AvroSerializer
from confluent_kafka.serialization import StringSerializer

DEFAULT_BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"
DEFAULT_SCHEMA_REGISTRY_URL = "http://127.0.0.1:8081"


class KafkaClient:
    """Wrap the Confluent Kafka Python library (which itself wraps librdkafka).

    Bootstrap servers and schema registry URL are read from environment
    variables ``KAFKA_BOOTSTRAP_SERVERS`` and ``KAFKA_SCHEMA_REGISTRY_URL``,
    falling back to the local-cluster defaults.
    """

    def __init__(
        self,
        bootstrap_servers: Optional[str] = None,
        schema_registry_url: Optional[str] = None,
    ) -> None:
        bootstrap = (
            bootstrap_servers
            or os.environ.get("KAFKA_BOOTSTRAP_SERVERS")
            or DEFAULT_BOOTSTRAP_SERVERS
        )
        registry = (
            schema_registry_url
            or os.environ.get("KAFKA_SCHEMA_REGISTRY_URL")
            or DEFAULT_SCHEMA_REGISTRY_URL
        )
        self.bootstrap_servers_dict = {"bootstrap.servers": bootstrap}
        self.schema_registry_dict = {"url": registry}
        self.config = dict(self.bootstrap_servers_dict)
        self.schema_registry_client: Optional[SchemaRegistryClient] = None
        self.serializer: Optional[AvroSerializer] = None
        self.producer: Optional[SerializingProducer] = None
        self.consumer_client: Optional[Consumer] = None
        self.admin_client = AdminClient(self.bootstrap_servers_dict)
        self.produced = 0
        self.idempotence = 1

    @staticmethod
    def delivery_report(err, msg) -> None:
        """Confluent Kafka delivery callback — logs failures to stdout."""
        if err is not None:
            print(f"Message delivery failed: {err}")

    @staticmethod
    def throttle_report() -> None:
        print("throttling")

    def produce(self, payload: dict, schema_str: str, topic: str, flush: bool = False) -> None:
        """Produce a single Avro-serialized record to ``topic``.

        ``payload`` must contain ``key`` and ``value`` entries. When ``flush``
        is True the call becomes synchronous (useful for tests).
        """
        self.produced += 1

        if self.schema_registry_client is None:
            self.schema_registry_client = SchemaRegistryClient(self.schema_registry_dict)

        if self.producer is None:
            self.serializer = AvroSerializer(
                schema_str=schema_str,
                schema_registry_client=self.schema_registry_client,
            )
            self.producer = SerializingProducer(
                {
                    "bootstrap.servers": self.bootstrap_servers_dict["bootstrap.servers"],
                    "key.serializer": StringSerializer("utf_8"),
                    "value.serializer": self.serializer,
                    "enable.idempotence": self.idempotence,
                    "throttle_cb": self.throttle_report,
                }
            )

        if self.produced % 10000 == 0:
            self.producer.poll(0)

        self.producer.produce(
            topic=topic,
            key=payload["key"],
            value=payload["value"],
            on_delivery=self.delivery_report,
        )

        if flush:
            self.producer.flush()

    def create_or_update_topic(self, topic: str, schema_str: str) -> None:
        """Create ``topic`` in the broker and register its Avro schema.

        Schema evolution (rejecting incompatible changes, etc.) is not yet
        wired up — this method only creates new topics and schema versions.
        """
        topic_list = [NewTopic(topic, num_partitions=3, replication_factor=1)]
        self.admin_client.create_topics(new_topics=topic_list, validate_only=False)

        if self.schema_registry_client is None:
            self.schema_registry_client = SchemaRegistryClient(self.schema_registry_dict)
        self.schema_registry_client.register_schema(
            subject_name=topic,
            schema=Schema(schema_str=schema_str, schema_type="AVRO"),
        )

    def consumer(self, group_id: str) -> Consumer:
        """Construct (or return the cached) ``confluent_kafka.Consumer`` for ``group_id``."""
        config = dict(self.config)
        config["group.id"] = group_id
        self.consumer_client = Consumer(config)
        return self.consumer_client

    def get_metadata(self, topic: str) -> Tuple[dict, dict]:
        """Return (topics, brokers) metadata for ``topic`` from the broker."""
        cluster_metadata = self.admin_client.list_topics(timeout=10, topic=topic)
        return cluster_metadata.topics, cluster_metadata.brokers
