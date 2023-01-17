


import pandas as pd
import pyspark.sql.functions as F
from pyspark.sql import Window
from pyspark.sql.functions import pandas_udf, PandasUDFType, udf
from pyspark.sql.types import FloatType, StructType, StructField, IntegerType, StringType

import os
from pyspark.sql import SparkSession
import pyspark.sql.functions as F


def spark_context(app_name: str) -> None:

    jars_packages = [
        # For Kafka access
        "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
        "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
        "org.apache.kafka:kafka-clients:3.3.1",
    ]

    repos = [
        "https://packages.confluent.io/maven",
    ]

    jars = ",".join(jars_packages)
    repos = ",".join(repos)

    args = f"--packages {jars} --repositories {repos} pyspark-shell"
    os.environ["PYSPARK_SUBMIT_ARGS"] = args

    config = {
        "spark.sql.execution.arrow.pyspark.enabled": "true",
        "spark.sql.session.timeZone": "UTC",
        "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint",
    }

    spark = SparkSession.builder.master("local[*]").appName(app_name)

    for key, value in config.items():
        spark.config(key, value)
    return spark.getOrCreate()



spark = spark_context("TumblingWindows")




df = spark.createDataFrame(
  [(1, "2021-04-01", 10, -30),
   (1, "2021-03-01", 10, 20),
   (1, "2021-02-01", 10, -1),
   (1, "2021-01-01", 10, 10),
   (1, "2020-12-01", 10, 5),
   (1, "2021-04-01", 20, -5),
   (1, "2021-03-01", 20, -4),
   (1, "2021-02-01", 20, -3),
   (2, "2021-03-01", 10, 5),
   (2, "2021-02-01", 10, 6),
  ], 
  StructType([
    StructField("csecid", StringType(), True), 
    StructField("date", StringType(), True), 
    StructField("analystid", IntegerType(), True), 
    StructField("revisions_improved", IntegerType(), True)
  ]))

### Baseline
@pandas_udf(FloatType(), PandasUDFType.GROUPED_AGG)
def method2(analyst: pd.Series, revisions: pd.Series) -> float:
  df = pd.DataFrame({
    'analyst': analyst,
    'revisions': revisions
  })
  return df.groupby('analyst').last()['revisions'].sum() / df.groupby('analyst').last()['revisions'].abs().sum()




### Method 3
from typing import List

@udf(FloatType())
def method3(analyst: List[int], revisions: List[int]) -> float:
  df = pd.DataFrame({
    'analyst': analyst,
    'revisions': revisions
  })
  return float(df.groupby('analyst').last()['revisions'].sum() / df.groupby('analyst').last()['revisions'].abs().sum())

days = lambda x: x*60*60*24
w = Window.partitionBy('csecid').orderBy(F.col('date').cast('timestamp').cast('long')).rangeBetween(-days(100), 0)


(df
.withColumn('new_col', method2(F.col('analystid'), F.col('revisions_improved')).over(w))

.withColumn('analyst_win', F.collect_list("analystid").over(w))
.withColumn('revisions_win', F.collect_list("revisions_improved").over(w))

.withColumn('method3', method3(F.collect_list("analystid").over(w), 
                               F.collect_list("revisions_improved").over(w)))
.orderBy("csecid", "date", "analystid")
.show(truncate=False))