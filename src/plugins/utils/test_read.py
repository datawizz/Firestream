from pyspark.sql import SparkSession

spark = SparkSession.builder.appName("ReadParquet").getOrCreate()

df = spark.read.parquet("/workspace/src/lib/python/etl_lib/utils/output_file.parquet")


df = df.selectExpr(
    "timestamp",
    "CAST(key AS STRING)",
    "CAST(value AS STRING)"
)

df.show(20, truncate=False)
