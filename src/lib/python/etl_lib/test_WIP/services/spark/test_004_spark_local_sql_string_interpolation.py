from numpy import source
from pyspark.sql import SparkSession, DataFrame
from pyspark.sql.types import *
from pyspark.sql.functions import expr
from pyspark.sql.functions import when, col
from pyspark.sql.types import *

from dataclasses import dataclass
from typing import List


def test_string_interpolation():

    spark = SparkSession.builder.appName("SparkByExamples.com").getOrCreate()
    data = [
        ("James", "M", 60000),
        ("Michael", "M", 70000),
        ("Robert", None, 400000),
        ("Maria", "F", 500000),
        ("Jen", "", None),
    ]

    columns = ["name", "gender", "salary"]
    # df = spark.createDataFrame(data=data, schema=columns)
    # df.show()

    my_schema = StructType(
        [
            StructField("id", LongType()),
            StructField(
                "country",
                StructType(
                    [
                        StructField("name", StringType()),
                        StructField("capital", StringType()),
                    ]
                ),
            ),
            StructField("currency", StringType()),
        ]
    )
    l = [
        (1, {"name": "Italy", "capital": "Rome"}, "euro"),
        (2, {"name": "France", "capital": "Paris"}, "euro"),
        (3, {"name": "Japan", "capital": "Tokyo"}, "yen"),
    ]
    df = spark.createDataFrame(l, schema=my_schema)

    gender1 = "Male"
    gender2 = "Female"
    _col = "country.name"

    df4 = df.select(
        col(f"{_col}"),
        expr(
            f"""
            CASE
            WHEN ({_col} = 'Italy') THEN '{gender1}'
            WHEN {_col} = 'F' AND {_col} != '' THEN '{gender2}'
            WHEN {_col} IS NULL THEN ''
            ELSE {_col} END
            """
        ).alias("new_gender"),
    )

    df4.show()


if __name__ == "__main__":
    test_string_interpolation()
