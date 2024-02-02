from tinsel import struct, transform
from dataclasses import dataclass
from pyspark.sql import SparkSession
from etl_lib.services.spark.client import SparkClient
from dataclasses_avroschema import AvroModel
from typing import List, Optional, Dict
import pandas as pd
import json
from datetime import datetime


from etl_lib.services.spark.client import SparkClient
from etl_lib import DataModel, DataFactory
from etl_lib.services.kafka.client import KafkaClient


def test_spark_client_s3_read_write():

    from etl_lib.services.spark.client import SparkClient

    spark = SparkClient(app_name="a_test").spark_session

    df = spark.read.csv("s3a://data/example/cities.csv")

    df.printSchema()
    print(df.count())

    df.write.format("parquet").save("s3a://data/destination")

    spark.stop()


topic = "test_end_to_end_batch2"


@struct
@dataclass
class UserInfo(AvroModel):
    hobby: List[str]
    last_seen: Optional[int]
    pet_ages: Dict[str, int]


@struct
@dataclass
class User2(DataModel):
    login: str
    age: int
    active: bool
    key: str
    timestamp: datetime
    # user_info: UserInfo = None #TODO fix "fake()" method to create nested objects as pandas series


def test_end_to_end_batch(model=User2, topic=topic):
    """
    1. Select a nested dataclass (complex object)
    2. Generate some random data from the class definition
    3. Write the Pandas Dataframe to Kafka using Spark
    4. Read the topic in Kafka using Spark
    5. Return the spark datafrae as a pandas dataframe using PyArrow

    Create some random data
    Write that data to a topic
    Read that data from the topic
    Assert that the read data matches the generated data
    """

    spark_schema = transform(model)

    schema_str = model.avro_schema()

    schema_str = json.loads(schema_str)
    schema_str.update({"name": "value"})
    schema_str = json.dumps(schema_str)

    print(schema_str)
    pdf = pd.DataFrame([model.fake() for i in range(100)])

    print(pdf)

    # pdf["value"] = pdf

    client = SparkClient()
    sc = client.context
    # Create a Spark DataFrame from a Pandas DataFrame using Arrow
    sdf = sc.createDataFrame(pdf)

    header = DataFactory(model=model, streaming=False)

    client.df_to_kafka(
        df=sdf,
        topic=header.topic,
        schema_str=header.avro_schema_str,
        streaming=header.streaming,
    )

    # TODO make sure the schema is set in Kafka when writing by this method.
    # Think about using the admin interface to create the topic beforehand.

    df2 = client.df_from_kafka(
        topic=header.topic,
        streaming=header.streaming,
    )

    # Convert the Spark DataFrame back to a Pandas DataFrame using Arrow
    pdf2 = df2.select(["key", "value"]).toPandas()
    pdf2 = pd.DataFrame(pdf2["value"].to_list(), index=pdf2.index)

    pdf2 = pdf2.loc[pdf.index, :]

    print(pdf)
    print(pdf2)

    assert pd.testing.assert_frame_equal(pdf, pdf2)


def test_topic_creation(model=User2):
    """ """
    c = KafkaClient()
    print(
        c.create_or_update_topic(
            topic=model.__name__, schema_str=model.get_schema("avro")()
        )
    )

    # print(c.topic_metadata())


if __name__ == "__main__":

    test_topic_creation()
    # SCHEMA NOT LOADING!

    test_end_to_end_batch()
