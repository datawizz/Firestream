import abc
import json
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, Generic, List, TypeVar

from pydantic import BaseModel
from pydantic_avro.base import AvroBase
from pydantic_spark.base import SparkBase
from pyspark.sql import DataFrame as SparkDataFrame


class SupportedSchemaTypes(Enum):
    """Systems where a compatible schema is supported by this library."""

    AVRO = "AVRO"
    SPARK = "SPARK"
    JSON = "JSON"
    SOLR = "SOLR"
    PETASTORM = "PETASTORM"
    DELTA = "DELTA"


_TDataModel = TypeVar("_TDataModel", bound="DataModel")


class DataModel(SparkBase, AvroBase, BaseModel, Generic[_TDataModel], metaclass=abc.ABCMeta):
    """The DataModel represents Data in motion, at rest, and in transit."""

    def get_schema_dict(self, model: BaseModel) -> Dict[str, Any]:
        """Recursively resolve the schema dictionary including nested DataModels."""
        schema = type(model).model_json_schema()
        for field, value in schema.get("properties", {}).items():
            if "$ref" in value:
                child_model = type(model).__annotations__[field]
                value.update(self.get_schema_dict(child_model()))
        return schema

    def json_schema(self) -> str:
        """Return the JSON schema for this model, including nested references."""
        return json.dumps(self.get_schema_dict(model=self))

    def base_types(self) -> List[type]:
        """Return the list of base scalar types this library natively supports."""
        return [str, int, float, bool, datetime, timedelta]

    def get_schema(self, format: SupportedSchemaTypes = SupportedSchemaTypes.JSON) -> str:
        """Return the schema of this model in the requested format."""
        schema_format = SupportedSchemaTypes(format)
        if schema_format == SupportedSchemaTypes.JSON:
            return self.json_schema()
        if schema_format == SupportedSchemaTypes.AVRO:
            return self.avro_schema()
        raise NotImplementedError(f"Schema type {format} not supported")

    @classmethod
    @abc.abstractmethod
    def make(cls, context: Any, df: Any = None) -> SparkDataFrame:
        """Build a SparkDataFrame for this model from a DataContext.

        Subclasses must implement this. Calling it on the base class raises
        NotImplementedError to signal the contract is not yet wired up.
        """
        raise NotImplementedError("DataModel.make must be implemented by subclasses")
