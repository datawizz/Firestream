from etl_lib.services.spark.client import SparkClient

spark = SparkClient("test").session

import pandas as pd

from pyspark.sql.functions import pandas_udf
import numpy as np
import pandas as pd

import pandas as pd
from pyspark.sql.functions import pandas_udf, ceil


def test_spark_with_pandas_udf():

    # Generate a Pandas DataFrame
    pdf = pd.DataFrame(np.random.rand(100, 3))

    # Create a Spark DataFrame from a Pandas DataFrame using Arrow
    df = spark.createDataFrame(pdf)

    # Convert the Spark DataFrame back to a Pandas DataFrame using Arrow
    result_pdf = df.select("*").toPandas()

    @pandas_udf("col1 string, col2 long")
    def func(s1: pd.Series, s2: pd.Series, s3: pd.DataFrame) -> pd.DataFrame:
        s3["col2"] = s1 + s2.str.len()
        return s3

    # Create a Spark DataFrame that has three columns including a sturct column.
    df = spark.createDataFrame(
        [[1, "a string", ("a nested string",)]],
        "long_col long, string_col string, struct_col struct<col1:string>",
    )

    df.printSchema()
    # root
    # |-- long_column: long (nullable = true)
    # |-- string_column: string (nullable = true)
    # |-- struct_column: struct (nullable = true)
    # |    |-- col1: string (nullable = true)

    df.select(func("long_col", "string_col", "struct_col")).printSchema()
    # |-- func(long_col, string_col, struct_col): struct (nullable = true)
    # |    |-- col1: string (nullable = true)
    # |    |-- col2: long (nullable = true)

    df = spark.createDataFrame([(1, 21), (2, 30)], ("id", "age"))

    def filter_func(iterator):
        for pdf in iterator:
            yield pdf[pdf.id == 1]

    df.mapInPandas(filter_func, schema=df.schema).show()

    df = spark.createDataFrame(
        [(1, 1.0), (1, 2.0), (2, 3.0), (2, 5.0), (2, 10.0)], ("id", "v")
    )

    def normalize(pdf):
        v = pdf.v
        return pdf.assign(v=(v - v.mean()) / v.std())

    df.groupby("id").applyInPandas(normalize, schema="id long, v double").show()


if __name__ == "__main__":
    test_spark_with_pandas_udf()
