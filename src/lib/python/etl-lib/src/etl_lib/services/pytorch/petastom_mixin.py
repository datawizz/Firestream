import numpy as np
from pyspark.sql import SparkSession
from pyspark.sql.types import IntegerType

from petastorm.codecs import ScalarCodec, CompressedImageCodec, NdarrayCodec
from petastorm.etl.dataset_metadata import materialize_dataset
from petastorm.unischema import dict_to_spark_row, Unischema, UnischemaField

from typing import get_type_hints

from etl_lib.source import DataSource
from etl_lib.model import DataModel
from etl_lib.services.spark.client import SparkClient

from dataclasses import dataclass

# The schema defines how the dataset schema looks like
HelloWorldSchema = Unischema(
    "HelloWorldSchema",
    [
        UnischemaField("_id", np.int32, (), ScalarCodec(IntegerType()), False),
        UnischemaField(
            "array_4d", np.uint8, (None, 128, 30, None), NdarrayCodec(), False
        ),
    ],
)


@dataclass
class A_Petastorm_Model(DataModel):
    """
    Extend a data model to return the appropriate schema
    """

    _id: np.int32
    array_4d: np.uint8

    @classmethod
    def fields(cls, implementation: str) -> list:
        """
        Return fields in self encoded in the appropriate schema
        """
        print(cls.rendered_schema)

        if implementation == "petastorm":
            for _field, _type in get_type_hints(cls).items():
                print(_field, _type)

    @classmethod
    def as_petastorm_schema(cls):
        # The schema defines how the dataset schema looks like
        return Unischema(name=cls.__name__, fields=[x for x in cls.fields("petastorm")])


def row_generator(x):
    """Returns a single entry in the generated dataset. Return a bunch of random values as an example."""
    return {
        "id": x,
        "array_4d": np.random.randint(0, 255, dtype=np.uint8, size=(4, 128, 30, 3)),
    }


def generate_petastorm_dataset(
    output_url="file:///workspace/.data/hello_world_dataset",
):
    rowgroup_size_mb = 256

    spark = SparkClient()
    sc = spark.spark_context
    spark = spark.session

    # Wrap dataset materialization portion. Will take care of setting up spark environment variables as
    # well as save petastorm specific metadata
    rows_count = 10
    with materialize_dataset(spark, output_url, HelloWorldSchema, rowgroup_size_mb):

        rows_rdd = (
            sc.parallelize(range(rows_count))
            .map(row_generator)
            .map(lambda x: dict_to_spark_row(HelloWorldSchema, x))
        )

        spark.createDataFrame(rows_rdd, HelloWorldSchema.as_spark_schema()).coalesce(
            10
        ).write.mode("overwrite").parquet(output_url)


if __name__ == "__main__":
    # generate_petastorm_dataset()
    A_Petastorm_Model.fields("petastorm")
