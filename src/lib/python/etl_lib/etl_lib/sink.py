import os
import json
from requests import Session
from pyspark.sql import DataFrame as SparkDataFrame
import pandas as pd
import time
from typing import Union, List, Generator
from enum import Enum
from datetime import datetime, timedelta


from etl_lib.model import DataModel
from etl_lib.context import DataContext


from etl_lib.services.kafka.client import KafkaClient
import abc

from etl_lib.source import BrownianMotion_DataSource


class DataSink(DataContext, metaclass=abc.ABCMeta):
    """
    The Abstract Base Class for writing data
    """

    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)

    @abc.abstractmethod
    def sink(self) -> None:
        """
        Write out the data
        """
        ...


class Local_DataSink(DataSink):
    """
    Sink a Spark DataFrame locally
    Useful for development, training, and testing

    Is called when cache=True in the DataFactory
    """

    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)

        # Petastorm initialize


class Console_DataSink(DataSink):
    """
    Write data to the STOUT
    Useful in testing and development
    """

    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)

    def sink(self) -> None:
        """
        Write the DataFrame to the console along with the schema

        Log it if logging is enabled
        """

        if not self.streaming:
            self.spark_df.show()
            self.spark_df.printSchema()
        else:
            raise ValueError("Not implemented")

        return None


# class Kafka_DataSink(DataSink):
#     """
#     A data sink that is backed by Kafka
#     Encodes in Avro by default
#     """

#     def __init__(self, **kwargs) -> None:

#         super().__init__(**kwargs)

#         self.kafka_client = KafkaClient()
#         self.topic = self.model.__name__
#         self.partitions = self.model.partitions()

#     def sink(self):
#         """
#         Write Spark DataFrame to Kafka using the configured encoding
#         """

#         self.spark_client.df_to_kafka(
#             df=self.spark_df, topic=self.topic, schema=self.model.get_schema("avro")
#         )
#         self.kafka_client = KafkaClient()

#         # Reformat schema registry location for ABRiS
#         self.schema_registry_config = {
#             "schema.registry.url": self.kafka_client.schema_registry_dict.get("url")
#         }
#     @staticmethod
#     def __to_avro(col, config):
#         """
#         avro serialize
#         :param col (PySpark column / str): column name "key" or "value"
#         :param config (za.co.absa.abris.config.ToAvroConfig): abris config, generated from abris_config helper function
#         :return: PySpark Column
#         """
#         jvm_gateway = SparkContext._active_spark_context._gateway.jvm
#         abris_avro = jvm_gateway.za.co.absa.abris.avro

#         return Column(abris_avro.functions.to_avro(_to_java_column(col), config))

#     @staticmethod
#     def __to_avro_abris_config(
#         config_map: dict, topic: str, is_key: bool, schema_str: str
#     ):
#         """
#         Create to avro abris config with a schema url

#         :param config_map (dict[str, str]): configuration map to pass to the serializer, ex: {'schema.registry.url': 'http://localhost:8081'}
#         :param topic (str): kafka topic
#         :param is_key (bool): boolean
#         :return: za.co.absa.abris.config.ToAvroConfig
#         """
#         jvm_gateway = SparkContext._active_spark_context._gateway.jvm
#         scala_map = jvm_gateway.PythonUtils.toScalaMap(config_map)

#         return (
#             jvm_gateway.za.co.absa.abris.config.AbrisConfig.toConfluentAvro()
#             .provideAndRegisterSchema(schema_str)
#             .usingTopicRecordNameStrategy(topic)
#             .usingSchemaRegistry(scala_map)
#         )

#     #    @staticmethod
#     #    def schema_from_class(cls):
#     #
#     #        """
#     #        Call the tinsel library to transform a dataclass to a Spark SQL Dataframe schema
#     #        """
#     #        # TODO assert cls has been decorated with @struct assert cls
#     #        return transform(cls)

#     def df_to_kafka(self, df: DataFrame, topic: str, schema_str: str, streaming=False):
#         """
#         Given a spark dataframe, write that dataframe to Kafka in Avro format
#         using default spark with AbriS schema registry bridge

#         The write method also needs to handle deduplicating the data that it is writing.
#             This needs water marking.


#         # TODO TB-237 ensure Arrow is used
#         # TODO TB-237 ABRiS does not set the schema in the schema registry.

#         """

#         # Use the Python admin client to create the Schema for the topic if needed.
#         # Throw an error if the provided schema cannot be "evolved"

#         config = self.__to_avro_abris_config(
#             config_map=self.schema_registry_config,
#             topic=topic,
#             is_key=False,  # Indicates that "value" should be encoded, not "key" field of the tuple
#             schema_str=schema_str,
#         )

#         # Get all columns and package as a top level structure
#         output = df.withColumn(
#             colName="value", col=struct([col(c).alias(c) for c in df.columns])
#         )
#         output = output.withColumn(colName="key", col=col("key").cast("string"))

#         # Serialize as confluent Avro
#         output = output.withColumn(
#             "value",
#             self.__to_avro(col="value", config=config).alias("value"),
#         )

#         output = output.select(["key", "value"])

#         output.show()
#         output.printSchema()

#         # output.printSchema()
#         # output.show(truncate=True)
#         # Set non-nullable to prevent warnings
#         # output.schema["value"].nullable = False
#         # output.schema["key"].nullable = False
#         # output = output.withColumn("b", output.key.isNull())

#         print("Writing to Kafka")
#         # output.printSchema()
#         # output.show(truncate=True)

#         if not streaming:
#             # Its a batch!
#             output.write.format("kafka").option(
#                 "kafka.bootstrap.servers",
#                 self.kafka_client.bootstrap_servers_dict.get("bootstrap.servers"),
#             ).option("topic", topic).save()
#         else:
#             # Its a stream!
#             output.writeStream.format("kafka").outputMode("append").option(
#                 "kafka.bootstrap.servers",
#                 self.kafka_client.bootstrap_servers_dict.get("bootstrap.servers"),
#             ).option("topic", topic).start().awaitTermination()


class TestSink(Console_DataSink, BrownianMotion_DataSource):
    """
    Takes as input a hash of the data and seed
    Fails if they don't match once generated using TestSource (geometric brownian motion?)
    """

    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)

    def sink(self):
        """
        Takes the Spark DataFrame in self and compares it to known good data if specified
        """

        self.spark_df


class S3_DataSink(DataSink):
    """
    Includes configurations for writing data to S3

    Overrides the AWS S3 settings to point to the localstack cluster.

    Partitions based on the specified fields and time.

    S3 abstracted with a Hadoop library behaves almost like a file system.
    Listing files scales liniarly with the number of files which is a problem.
    Partitioning by as many categorical columns as possible
    and by time in yyyy/mm/dd/hh/mm format also drastically reduces

    https://www.oak-tree.tech/blog/spark-kubernetes-minio

    """
    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)
        # use DataHeader to form a fully qualified name of the sharded path
        base_path = os.environ.get("S3_ENDPOINT_URL")
        _url = f"{base_path}/{self.local_path}"


    @abc.abstractmethod
    def sink(self) -> None:
        """
        Write out the data
        """
        ...


    


class GoogleCloudStorage(DataSink):
    """
    https://cloud.google.com/dataproc/docs/concepts/connectors/cloud-storage
    TODO
    """
