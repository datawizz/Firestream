from pyspark.sql import DataFrame as SparkDataFrame
from pyspark.sql import functions as F
from pyspark.sql.types import TimestampType

from etl_lib.context import DataContext


def stream_stream_bucket_join(
    left_df: SparkDataFrame, right_df: SparkDataFrame, context: DataContext
) -> SparkDataFrame:
    """Join two event-time streams via shared time buckets.

    A naive stream-stream equi-join on event time can explode into a Cartesian
    product. Bucketing both sides into windows of ``min(context.intervals)``
    converts the problem into two stream/static joins against the bucket frame,
    avoiding the explosion.

    See:
      * http://zachmoshe.com/2016/09/26/efficient-range-joins-with-spark.html
      * https://stackoverflow.com/q/66209588

    The bucket frame spans ``context.start`` to ``context.end``.
    """
    interval_seconds = str(int(min(context.intervals).total_seconds()))

    left_df = left_df.select(
        F.unix_timestamp(F.column("event_time"), "yyyy/MM/dd HH:mm:ss")
        .cast(TimestampType())
        .alias("event_time"),
        F.struct(*[c for c in left_df.columns]).alias("left_df"),
    )
    right_df = right_df.select(
        F.unix_timestamp(F.column("event_time"), "yyyy/MM/dd HH:mm:ss")
        .cast(TimestampType())
        .alias("event_time"),
        F.struct(*[c for c in right_df.columns]).alias("right_df"),
    )

    buckets = left_df.agg(
        F.expr(
            f"""transform(
                    sequence(date_trunc('second', to_timestamp('{context.start.isoformat()}')),
                            date_trunc('second', to_timestamp('{context.end.isoformat()}')),
                            interval {interval_seconds} second
                    ),
                    x -> struct(x as start, x + interval {interval_seconds} second as end)
                )
            """
        ).alias("buckets")
    ).select(F.explode("buckets").alias("window"))

    left_bucketed = (
        buckets.join(
            left_df,
            (F.col("event_time") >= F.col("window.start"))
            & (F.col("event_time") < F.col("window.end")),
            "inner",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("left_df")).alias("left_df"))
    )
    right_bucketed = (
        buckets.join(
            right_df,
            (F.col("event_time") >= F.col("window.start"))
            & (F.col("event_time") < F.col("window.end")),
            "inner",
        )
        .groupBy("window")
        .agg(F.collect_list(F.col("right_df")).alias("right_df"))
    )

    return left_bucketed.join(right_bucketed, left_bucketed.window == right_bucketed.window)
