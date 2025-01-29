import numpy as np
from pyspark.sql import SparkSession
from pyspark import SparkContext
from pyspark.sql import functions as F
from pyspark.sql.window import Window


def rest_cul_dist():

    sc = SparkContext("local", "Example")
    spark = SparkSession(sc)

    a = sc.parallelize([(float(x),) for x in np.random.normal(0, 1, 1000)]).toDF(["X"])
    a.limit(5).show()

    win = Window.orderBy("X")

    output = (
        a.withColumn("cumulative_probability", F.percent_rank().over(win))
        .withColumn("X", F.round(F.col("X"), 1))
        .groupBy("X")
        .agg(
            F.max("cumulative_probability").alias("cumulative_probability"),
            F.count("*").alias("my_count"),
        )
    )

    output.limit(5).show()

    output = (
        a.withColumn("cumulative_probability", F.percent_rank().over(win))
        .withColumn("X", F.round(F.col("X"), 1))
        .groupBy("X")
        .agg(
            F.max("cumulative_probability").alias("cumulative_probability"),
            F.count("*").alias("my_count"),
        )
        .withColumn(
            "cumulative_probability", F.lead(F.col("cumulative_probability")).over(win)
        )
        .fillna(1, subset=["cumulative_probability"])
    )

    output.limit(5).show()
