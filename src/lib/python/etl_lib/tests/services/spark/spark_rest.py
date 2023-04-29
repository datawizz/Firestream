import requests
import json
from pyspark.sql.functions import udf, col, explode
from pyspark.sql.types import (
    StructType,
    StructField,
    IntegerType,
    StringType,
    ArrayType,
)
from pyspark.sql import Row


def test():

    schema = StructType(
        [
            StructField("Count", IntegerType(), True),
            StructField("Message", StringType(), True),
            StructField("SearchCriteria", StringType(), True),
            # StructField("Results", StringType(), True),
            StructField(
                "Results",
                ArrayType(
                    StructType(
                        [
                            StructField("Make_ID", IntegerType()),
                            StructField("Make_Name", StringType()),
                        ]
                    )
                ),
            ),
        ]
    )

    @udf(returnType=schema)
    def executeRestApi(url):
        #
        headers = {"content-type": "application/json"}

        # data = []

        with requests.get(url, headers=headers, stream=False) as req:

            # print(req.json())
            data = req.json()

            # for chunk in req.iter_content(chunk_size=4096 * 64):
            #     print(chunk)
            #     data.append(chunk.json())

        return data

    # udf_executeRestApi = udf(lambda a, b, c, d: executeRestApi(a, b, c, d), schema)

    from pyspark.sql import Row
    from etl_lib.services.spark.client import SparkClient

    c = SparkClient(app_name="test")
    spark = c.session

    # body = json.dumps({})

    RestApiRequestRow = Row("url")

    request_df = spark.createDataFrame(
        [
            RestApiRequestRow(
                "https://vpic.nhtsa.dot.gov/api/vehicles/getallmakes?format=json"
            )
        ]
    )

    result_df = request_df.withColumn("result", executeRestApi(col("url")))

    result_df.show(truncate=True)
    result_df.printSchema()

    df = result_df.select(explode(col("result.Results")).alias("results"))
    df.show()
    df.printSchema()
    # df.select(collapse_columns(df.schema)).show()
    # df.show()
    # df.printSchema()


if __name__ == "__main__":
    test()