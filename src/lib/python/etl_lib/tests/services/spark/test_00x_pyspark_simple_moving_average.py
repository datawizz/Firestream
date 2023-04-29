from pyspark.sql import functions as F
from pyspark.sql.window import Window

from etl_lib.services.spark.client import SparkClient


def test():

    # function to calculate number of seconds from number of days
    days = lambda i: i * 86400
    spark = SparkClient().session
    df = spark.createDataFrame(
        [
            (1, 17, "2017-03-10T15:27:18+00:00"),
            (1, 13, "2017-03-15T12:27:18+00:00"),
            (1, 25, "2017-03-18T11:27:18+00:00"),
        ],
        ["id", "dollars", "timestampGMT"],
    )
    df.show()
    df = df.withColumn("timestampGMT", df.timestampGMT.cast("timestamp"))
    df.show()
    # create window by casting timestamp to long (number of seconds)
    w = (
        Window.partitionBy("id")
        .orderBy(F.col("timestampGMT").cast("long"))
        .rangeBetween(-days(7), 0)
    )

    df = df.withColumn("rolling_average", F.avg("dollars").over(w))
    df.show()
