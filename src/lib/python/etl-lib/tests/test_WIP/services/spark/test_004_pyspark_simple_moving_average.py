from pyspark.sql import functions as F
from pyspark.sql.window import Window
import pandas as pd
from etl_lib.services.spark.client import SparkClient
from dataclasses import dataclass
from datetime import datetime
from etl_lib import DataModel
from pandas.testing import assert_series_equal


@dataclass
class Test_Data(DataModel):
    id: int
    dollars: int
    timestampGMT: datetime


interval = 7 * 86400  # 7 days in seconds


def create_in_pandas():

    data = [
        [
            (1, 17, "2017-03-10 15:27:18+00:00"),
            (2, 13, "2017-03-15 12:27:18+00:00"),
            (3, 25, "2017-03-18 11:27:18+00:00"),
        ],
        ["id", "dollars", "timestampGMT"],
    ]

    pdf = pd.DataFrame(data=data[0], columns=data[1])
    pdf["timestampGMT"] = pd.to_datetime(pdf["timestampGMT"])
    pdf.set_index(pdf["timestampGMT"], inplace=True)
    pdf["rolling_average"] = pdf.rolling("7d").dollars.mean()
    pdf.reset_index(inplace=True, drop=True)
    return pdf


def create_in_spark():

    # function to calculate number of seconds from number of days

    from pyspark.sql import functions as F
    from pyspark.sql.window import Window

    # function to calculate number of seconds from number of days
    days = lambda i: i * 86400

    spark = SparkClient().session
    df = spark.createDataFrame(
        data=create_in_pandas(), schema=Test_Data.as_spark_schema()
    )
    df = df.withColumn("timestampGMT", df.timestampGMT.cast("timestamp"))

    # create window by casting timestamp to long (number of seconds)
    w = Window.orderBy(F.col("timestampGMT").cast("long")).rangeBetween(-days(7), 0)

    df = df.withColumn("rolling_average", F.avg("dollars").over(w))

    return df


def test_spark_pandas():
    """
    Use the ETL Lib spark client with known good data to test initialization
    Perform some basic calculation over the data
    """

    pdf = create_in_pandas()
    sdf = create_in_spark()
    pdf2 = sdf.toPandas()

    assert_series_equal(pdf["rolling_average"], pdf2["rolling_average"])


if __name__ == "__main__":
    test_spark_pandas()
