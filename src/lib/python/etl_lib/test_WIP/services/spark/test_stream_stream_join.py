from pyspark.sql import functions as F, Row
from pyspark.sql.types import TimestampType
from pyspark import SparkContext
import pandas as pd
import pytz
from etl_lib import DataContext, DataModel
from etl_lib.services.spark.client import SparkClient


from datetime import timedelta


from pyspark.sql import DataFrame as SparkDataFrame


def stream_stream_bucket_join(
    left_df: SparkDataFrame, right_df: SparkDataFrame, context: DataContext
) -> SparkDataFrame:
    """

    Efficient Stream + Stream Joins by Bucketing

    Problem:
        Joins need to be made over event time data
        Event time processing over a stream requires use of Water Marks to deal with late data
        Event time joining, depending on the size of the dataset, can produce cartesian explosions (aka out of memory)

    Solution:
        Use the DataContext to build a temp table of the Cartesian explosion of start, end, and interval
        While this can be large, it is finite and thus changes the problem to two stream + static join, much faster!
        Use the temp table to join the two dataframes and return the result.
        Every column present in the source dataframes will be added to a list of sctructs, aka lossless

    Limitation:
        Spark will allow TimestampType up to fractions of a second. ??? how much?


    Steps:

        1. Define the interval to group records by
        2. Group the rows contained in each interval to an array of structs
        3. output the grouped df


    """
    client = context.spark_client

    smallest_interval = min(context.intervals)

    _interval = str(int(smallest_interval.total_seconds()))

    left_df = left_df.select(
        F.unix_timestamp(F.column("event_time"), "yyyy/MM/dd HH:mm:ss")
        .cast(TimestampType())
        .alias("event_time"),
        F.struct(*[c for c in left_df.columns]).alias("left_df"),
    )
    left_df.show()
    left_df.groupBy(F.window(F.col("event_time"), f"{_interval} seconds")).agg(
        F.collect_list("left_df")
    ).show(truncate=False)

    right_df = right_df.select(
        F.unix_timestamp(F.column("event_time"), "yyyy/MM/dd HH:mm:ss")
        .cast(TimestampType())
        .alias("event_time"),
        F.struct(*[c for c in right_df.columns]).alias("right_df"),
    )
    right_df.show()
    right_df.groupBy(F.window(F.col("event_time"), f"{_interval} seconds")).agg(
        F.collect_list("right_df")
    ).show(truncate=False)

    # Using a set of known buckets avoids calculating the cross product of the two streams
    # Since each stream can be compared to the buckets rather than eachother
    # http://zachmoshe.com/2016/09/26/efficient-range-joins-with-spark.html
    # https://stackoverflow.com/questions/66209588/apply-groupby-over-window-in-a-continuous-manner-pyspark

    buckets = left_df.agg(
        F.expr(
            f"""transform(
                    sequence(date_trunc('second', to_timestamp('{start.isoformat()}') ), 
                            date_trunc('second', to_timestamp('{end.isoformat()}') ), 
                            interval {_interval} second
                    ),
                    x -> struct(x as start, x + interval {_interval} second as end)
                )
        """
        ).alias("buckets")
    ).select(F.explode("buckets").alias("window"))

    buckets.show(truncate=False)

    left_df = (
        buckets.join(
            left_df,
            (F.col("event_time") >= F.col("window.start"))
            & (F.col("event_time") < F.col("window.end")),
            "inner",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("left_df")).alias("left_df"))
    )
    right_df = (
        buckets.join(
            right_df,
            (F.col("event_time") >= F.col("window.start"))
            & (F.col("event_time") < F.col("window.end")),
            "inner",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("right_df")).alias("right_df"))
    )
    left_df.show()
    right_df.show()

    out_df = left_df.join(right_df, (left_df.window == right_df.window))
    out_df.show(truncate=False)
    return out_df


def test_buckets():
    interval = [timedelta(seconds=60 * 60)]
    start = pd.to_datetime("2022-01-03 01:00", utc=True).astimezone(pytz.utc)
    end = pd.to_datetime("2022-01-03 17:00", utc=True).astimezone(pytz.utc)

    context = DataContext(model=DataModel, intervals=interval, start=start, end=end)

    df1 = context.spark_client.session.createDataFrame(
        [
            Row(event_time="2022/01/03 03:00:36", value=5.0),
            Row(event_time="2022/01/03 03:40:12", value=10.0),
            Row(event_time="2022/01/03 05:25:30", value=111.0),
        ]
    )
    df2 = context.spark_client.session.createDataFrame(
        [
            Row(event_time="2022/01/03 03:00:36", value=2.0),
            Row(event_time="2022/01/03 03:40:12", value=3.0),
            Row(event_time="2022/01/03 05:25:30", value=1.0),
        ]
    )

    stream_stream_bucket_join(left_df=df1, right_df=df2, context=context)


if __name__ == "__main__":
    test_buckets()
