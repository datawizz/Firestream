# Test vanilla spark + pyspark

from pyspark.sql import SparkSession

spark = SparkSession.builder.getOrCreate()

from datetime import datetime, date
import pandas as pd
from pyspark.sql import Row
import pyspark.sql.functions as F

INTERNAL_LISTENERS = "kafka-headless.kafka-cluster.svc.cluster.local:29092"


def test():
    configs = {
        "kafka_boostrap_servers": "kafka:localhost",
        "topic": "test_kafka_with_spark",
    }

    df = spark.createDataFrame(
        [
            Row(
                a=1,
                b=2.0,
                key="string1",
                d=date(2000, 1, 1),
                e=datetime(2000, 1, 1, 12, 0),
            ),
            Row(
                a=2,
                b=3.0,
                key="string2",
                d=date(2000, 2, 1),
                e=datetime(2000, 1, 2, 12, 0),
            ),
            Row(
                a=4,
                b=5.0,
                key="string3",
                d=date(2000, 3, 1),
                e=datetime(2000, 1, 3, 12, 0),
            ),
        ]
    )
    df.printSchema()
    df.show(truncate=False)

    from pyspark.sql.functions import array, col, lit, sort_array, struct

    # All columns but the meta data ones
    meta_cols = ["key"]
    columns_names = [x for x in df.columns if x not in meta_cols]

    df = df.withColumn("value", F.struct(*columns_names)).select(*meta_cols, "value")
    df.show()
    # # Write key-value data from a DataFrame to Kafka using a topic specified in the data
    # df.selectExpr(
    #     f"""{ configs.get("topic") } as topic""",
    #     "CAST(key AS STRING)",
    #     "CAST(value AS STRING)",
    # ).write.format("kafka").option(
    #     "kafka.bootstrap.servers", configs.get("kafka_boostrap_servers")
    # ).save()


if __name__ == "__main__":
    test()
