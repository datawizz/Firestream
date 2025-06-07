from pyspark.sql import functions as F, Row
from pyspark.sql.types import TimestampType
from pyspark import SparkContext
import pandas as pd
import pytz
from etl_lib import DataContext, DataModel
from etl_lib.services.spark.client import SparkClient


from datetime import timedelta

from typing import List
from pyspark.sql import DataFrame as SparkDataFrame


# def align_over_interval(
#     df: SparkDataFrame,
#     interval: timedelta,
#     cols_group: List[str],
#     col_time: str,
#     cols_agg_dict: dict,
# ):
#     """
#     Takes a spark DataFrame as input and returns a event_time partitioned DataFrame
#     Optionally grouping columns with an aggregate


#     Takes as input a Event Time partitioned DataFrame
#     Uses the context to find the set of intervals to align to.
#     Returns a DataFrame which is grouped into buckets which are structs of theunderlying rows,
#     repartitioned to the Interval Index.
#     """

#     interval_seconds = str(int(interval.total_seconds()))
#     print(interval_seconds)
#     df = df.withColumn("sample_interval", F.lit(interval_seconds))

#     df.show()

#     df_result = (
#         df.groupBy(
#             [F.col("sample_interval"), *[F.col(x) for x in cols_group]],
#             F.window(F.col(col_time), interval_seconds),
#         )
#     ).agg([v(F.col(k)).alias(k) for k, v in cols_agg_dict.items()])

#     return df_result


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

    Input:
        Left and Right DataFrames that each have a column named "event_time" which is Type TimestampType

    Steps:

        1. Define the interval to group records by
        2. Group the rows contained in each interval to an array of structs
        3. output the grouped df


    """
    client = context.spark_client

    smallest_interval = context.intervals[0]

    _interval = str(int(smallest_interval.total_seconds()))

    left_df = left_df.select(
        F.col("event_time"),
        F.struct(*[c for c in left_df.columns]).alias("left_df"),
    )
    left_df.show()
    left_df.groupBy(F.window(F.col("event_time"), f"{_interval} seconds")).agg(
        F.collect_list("left_df")
    ).show()

    right_df = right_df.select(
        F.col("event_time"),
        F.struct(*[c for c in right_df.columns]).alias("right_df"),
    )
    right_df.show()
    right_df.groupBy(F.window(F.col("event_time"), f"{_interval} seconds")).agg(
        F.collect_list("right_df")
    ).show()

    # Using a set of known buckets avoids calculating the cross product of the two streams
    # Since each stream can be compared to the buckets rather than eachother
    # http://zachmoshe.com/2016/09/26/efficient-range-joins-with-spark.html
    # https://stackoverflow.com/questions/66209588/apply-groupby-over-window-in-a-continuous-manner-pyspark

    buckets = left_df.agg(
        F.expr(
            f"""transform(
                    sequence(date_trunc('second', to_timestamp('{context.start.isoformat()}') ), 
                            date_trunc('second', to_timestamp('{context.end.isoformat()}') ), 
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
            "left",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("left_df")).alias("left_df"))
    )
    right_df = (
        buckets.join(
            right_df,
            (F.col("event_time") >= F.col("window.start"))
            & (F.col("event_time") < F.col("window.end")),
            "left",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("right_df")).alias("right_df"))
    )
    left_df.show()
    right_df.show()

    df = left_df.join(right_df, (left_df.window == right_df.window)).select(
        left_df.left_df, right_df.right_df, left_df.window
    )
    df.show()
    return df
