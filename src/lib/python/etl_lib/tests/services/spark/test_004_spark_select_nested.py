import json
from pyspark.sql.types import ArrayType, IntegerType, StructType, StructField
from pyspark.sql import functions as F

from etl_lib.services.spark.client import SparkClient
from etl_lib.core import DataFactory, DataModel
from dataclasses import dataclass


def test():

    sqlContext = SparkClient().session

    Source1 = json.dumps(
        [
            {"sku": "abc-abc-abc", "prod_id": "ss-23235", "salePrice": "2312"},
            {"sku": "xyz-xyz-xyz", "prod_id": "ss-13265", "salePrice": "8312"},
        ]
    )
    Source2 = json.dumps(
        [
            {"sku": "abc-abc-abc", "min_price": "678"},
            {"sku": "xyz-xyz-xyz", "min_price": "7655"},
        ]
    )

    df1 = sqlContext.read.json(Source1)
    df1.show()
    df2 = sqlContext.read.json(Source2)
    df2.show()
    df3 = (
        df1.join(df2, "sku", how="inner")
        .select(df1.sku, df1.prod_id, df1.salePrice, df2.min_price)
        .withColumn("price", F.to_json(F.struct(df1.salePrice, df2.min_price)))
        .drop("salePrice")
        .drop("min_price")
    )
    df4 = (
        df3.select("sku", "prod_id", "price")
        .withColumn("Output", F.to_json(F.struct("sku", "prod_id", "price")))
        .drop("sku")
        .drop("prod_id")
        .drop("price")
        .show(truncate=False)
    )
