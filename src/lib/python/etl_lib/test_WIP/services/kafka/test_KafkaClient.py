# import datetime
# from dataclasses import dataclass
# from uuid import uuid4
# from etl_lib.services.kafka.client import KafkaClient
# from etl_lib import DataFactory, DataModel


# @dataclass
# class test_model(DataModel):
#     name: str
#     favorite_number: str
#     favorite_color: str


# def test_produce_synchronous():

#     """
#     Test the topic against the docker dev env
#     """

#     topic = "test_avro_producer2"

#     client = KafkaClient()

#     for i in range(100):
#         payload = {
#             "key": str(datetime.datetime.now().timestamp() * 10**6),
#             "value": {
#                 "name": str(uuid4()),
#                 "favorite_number": str(uuid4()),
#                 "favorite_color": str(uuid4()),
#             },
#         }

#         client.produce(
#             schema_str=test_model.get_schema("spark"),
#             payload=payload,
#             topic=topic,
#             flush=True,
#         )
#         print(f"Produced {payload}")

#     return True


# if __name__ == "__main__":
#     test_produce_synchronous()


# from confluent_kafka.avro.cached_schema_registry_client import (
#     CachedSchemaRegistryClient,
# )


# def test_schema_registry():

#     sr = CachedSchemaRegistryClient(
#         {
#             "url": "http://localhost:8081",
#             # "ssl.certificate.location": "/path/to/cert",  # optional
#             # "ssl.key.location": "/path/to/key",  # optional
#         }
#     )

#     value_schema = sr.get_latest_schema("__consumer_offsets-value")[1]
#     key_schema = sr.get_latest_schema("__consumer_offsets-key")[1]

#     print(value_schema, key_schema)

#     from confluent_kafka.schema_registry.schema_registry_client import (
#         SchemaRegistryClient,
#     )

#     assert True == False

#     sr = SchemaRegistryClient(
#         {
#             "url": "http://localhost:8081",
#             # "ssl.certificate.location": "/path/to/cert",  # optional
#             # "ssl.key.location": "/path/to/key",  # optional
#         }
#     )
#     my_schema = sr.get_schema(schema_id=1)
#     print(my_schema.schema_str)


# if __name__ == "__main__":
#     test_schema_registry()


from etl_lib.services.kafka.client import KafkaClient
from etl_lib import DataModel
from dataclasses import dataclass
import datetime
from uuid import uuid4


@dataclass
class test_model(DataModel):
    name: str
    favorite_number: str
    favorite_color: str


def test_produce_synchronous():

    """
    Test the KafkaClient by producing Python dicts
    using the Python API and C# extension librdkafka
    with a schema defined by the ETL Library
    """

    model = test_model

    topic = model.__name__

    scheam_str = model.get_schema("avro")()

    client = KafkaClient()

    for i in range(100):
        payload = {
            "key": str(datetime.datetime.now().timestamp() * 10**6),
            "value": {
                "name": str(uuid4()),
                "favorite_number": str(uuid4()),
                "favorite_color": str(uuid4()),
            },
        }

        client.produce(topic=topic, schema_str=scheam_str, payload=payload, flush=True)
        print(f"Produced {payload}")

    return True


if __name__ == "__main__":
    test_produce_synchronous()
