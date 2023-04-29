import abc
import json


from pydantic import BaseModel
from pydantic_spark.base import SparkBase
from pydantic_avro.base import AvroBase
from polyfactory.factories.pydantic_factory import ModelFactory


from enum import Enum




from pyspark.sql import DataFrame as SparkDataFrame
from pyspark.sql.types import StructType

from datetime import datetime, timedelta

from typing import Dict, List

from typing import TypeVar, Generic


class SupportedSchemaTypes(Enum):
    """
    Define the systems where a compatible schema is supported
    """

    AVRO = "avro"
    SPARK = "spark"
    JSON = "json"
    SOLR = "solr"
    PETASTORM = "petastorm"




class DataModel(
    SparkBase, AvroBase, Generic[TypeVar("DataModel")], metaclass=abc.ABCMeta
):
    """

    The DataModel class is a wrapper of Python Dataclass
    It provides functions to convert the Python Dataclass to compatible schemas in different systems
    """


    @classmethod
    def get_schema(cls, format:SupportedSchemaTypes) -> str:
        """
        Return dictionary that describes the schema of whatever this class get inherited too,
        customized for the target system's expected schema format.
        """
        schema_format = SupportedSchemaTypes(format)

        if schema_format == SupportedSchemaTypes.AVRO:
            return cls.avro_schema()
        elif schema_format == SupportedSchemaTypes.SPARK:
            return cls.spark_schema()
        # elif type == SupportedSchemaTypes.JSON:
        #     return cls.as_json_schema()
        else:
            raise NotImplementedError(f"Schema type {format} not supported")



    # @classmethod
    # @abc.abstractmethod
    # def make(cls, context, df=None) -> SparkDataFrame:
    #     """
    #     The make method takes a DataContext object as input and
    #     returns a SparkDataFrmae
    #     """
    #     ...




    @classmethod
    def make_one(cls) -> Dict:
        """
        The make method takes a DataContext object as input and
        returns a SparkDataFrmae

        This basic method generates a random instance of the model
        """
        
        class _ModelFactory(ModelFactory[cls]):
            __model__ = cls
        
        return _ModelFactory.build()


    # @classmethod
    # def as_avro_schema(cls) -> str:
    #     """
    #     Return a Avro schema that matches all fields in "cls"
    #     Make this a class method to use outside of initialized objects
    #     """

    #     d = json.loads(cls.avro_schema())
    #     # d.update(
    #     #     {"name": f"{self.__name__}"}
    #     # )  # Patch the default name field for consistency with Kafka Schema Registry
    #     return json.dumps(d)

    # @classmethod
    # def as_spark_schema(cls) -> StructType:
    #     """
    #     Return a Spark Dataframe Schema that matches all fields in self
    #     # TODO take manual control over these datatypes to ensure proper resultion of nanosecond times
    #     """
    #     return tinsel.transform(cls)

    # @classmethod
    # def __name(cls):
    #     return cls.__name__

    # @staticmethod
    # @abc.abstractmethod
    # def valid_sources(cls) -> Dict[str, List]:
    #     """
    #     Define the valid inputs for this model.

    #     Return a mapping of SOURCE:str and a list of
    #     DataModels which could be used when building that source.
    #     This accounts for models which have multiple sources.
    #     """
    #     ...

