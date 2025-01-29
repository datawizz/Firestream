import abc
import json


from pydantic import BaseModel
from pydantic_spark.base import SparkBase
from pydantic_avro.base import AvroBase
# from polyfactory.factories.pydantic_factory import ModelFactory

from pydantic import BaseModel
from typing import Dict, Any


from enum import Enum




from pyspark.sql import DataFrame as SparkDataFrame
from pyspark.sql.types import StructType

from datetime import datetime, timedelta

from typing import Dict, List

from typing import TypeVar, Generic


class SupportedSchemaTypes(Enum):
    """
    Define the systems where a compatible schema is supported by this library
    """

    AVRO = "AVRO"
    SPARK = "SPARK"
    JSON = "JSON"
    SOLR = "SOLR"
    PETASTORM = "PETASTORM"
    DELTA = "DELTA"




class DataModel(
   SparkBase, AvroBase, BaseModel, Generic[TypeVar("DataModel")], metaclass=abc.ABCMeta 
):
    """
    The DataModel represents Data in motion, at rest, and in transit.
    """



    def get_schema_dict(self, model: BaseModel) -> Dict[str, Any]:
        """
        Recursively find the dictionary that describes the schema of the model among all of its dependencies.
        """
        schema = model.schema()

        for field, value in schema.get('properties', {}).items():
            if "$ref" in value:
                child_model_name = value["$ref"].split("/")[-1]
                child_model = model.__annotations__[field]
                value.update(self.get_schema_dict(child_model()))

        return schema



    def json_schema(self) -> str:
        """
        Return dictionary that describes the schema of whatever this class get inherited too,
        customized for the target system's expected schema format.
        """
        return json.dumps(self.get_schema_dict(model=self))


    def base_types(self) -> List:
        """
        Return a list of the base types that this class inherits from
        """
        return [str, int, float, bool, datetime, timedelta]



    def get_schema(self, format:SupportedSchemaTypes=SupportedSchemaTypes.JSON) -> str:
        """
        Returns the schema of the class wihout instantiating it
        """
        schema_format = SupportedSchemaTypes(format)

        if schema_format == SupportedSchemaTypes.JSON:
            return self.json_schema()
        elif schema_format == SupportedSchemaTypes.AVRO:
            return self.avro_schema()
        else:
            raise NotImplementedError(f"Schema type {format} not supported")


    # def get_dependencies(self) -> List[TypeVar("DataModel")]:
    #     """
    #     Returns a list of the dependencies of the class
    #     """
    #     _fields = self.__annotations__
    #     for _key, _value in _fields.items():
    #         print(_key, _value)
    #         if not _value in self.base_types() and issubclass(_value, TypeVar("DataModel")):
    #             # The 

    #     return self.__annotations__.values()


    # @classmethod
    # @abc.abstractmethod
    # def make(cls, context, df=None) -> SparkDataFrame:
    #     """
    #     The make method takes a DataContext object as input and
    #     returns a SparkDataFrmae
    #     """
    #     ...




    # @classmethod
    # def make_one(cls) -> Dict:
    #     """
    #     The make method takes a DataContext object as input and
    #     returns a SparkDataFrmae

    #     This basic method generates a random instance of the model
    #     """
        
    #     class _ModelFactory(ModelFactory[cls]):
    #         __model__ = cls
        
    #     return _ModelFactory.build()




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

