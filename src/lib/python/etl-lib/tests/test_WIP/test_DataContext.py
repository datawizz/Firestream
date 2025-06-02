from etl_lib import DataContext

from datetime import timedelta

from dataclasses import dataclass
from etl_lib import DataModel
from pandas.tseries.holiday import USFederalHolidayCalendar
from pandas.tseries.offsets import CustomBusinessDay
import pandas as pd


@dataclass
class some_model(DataModel):
    field: int


def create_index(context: DataContext):
    # TODO it might be more efficient to do this in Spark https://stackoverflow.com/questions/43141671/sparksql-on-pyspark-how-to-generate-time-series

    # Make use of business days calendar
    us_bd = CustomBusinessDay(calendar=USFederalHolidayCalendar())

    days = pd.date_range(start=context.start, end=context.end, freq=us_bd).tolist()

    windows = []
    for interval in context.intervals:
        frames = pd.interval_range(start=context.start, end=context.end, freq=interval)
        windows.extend(
            [(x.left.timestamp(), x.right.timestamp()) for x in frames.to_list()]
        )

    return windows


def test_create_index():
    c = DataContext(
        model=some_model,
        start="2022-01-01",
        end="2022-01-02",
        intervals=[
            timedelta(seconds=1),
            timedelta(seconds=5),
            timedelta(seconds=15),
            timedelta(seconds=30),
        ],
    )

    t = create_index(context=c)
    print(t)
    print(len(t))


def test_market_days():
    c = DataContext(
        model=some_model,
        start="2021-01-01",
        end="2022-01-02",
        intervals=[
            timedelta(seconds=1),
            timedelta(seconds=5),
            timedelta(seconds=15),
            timedelta(seconds=30),
        ],
    )
    _data = c.market_days()
    print(_data)


from datetime import timedelta

from etl_lib.context import DataHeader, DataIndex, DataFooter

from etl_lib import DataContext

from etl_lib.services.spark.client import SparkClient


def test():

    context = DataContext(
        model=DataHeader,
        start="2022-01-01T15:32:52.192548651",
        end="2022-01-31",
        intervals=[timedelta(seconds=(x + 10) * (x + 1)) for x in range(10)],
        spark_client=SparkClient(),
    )

    df = DataHeader.make(context)
    df.show(truncate=False)


if __name__ == "__main__":
    test()


if __name__ == "__main__":

    # test_create_index()
    test_market_days()




# Use the Tinsel library https://github.com/benchsci/tinsel to generate a Spark dataframe schema


from dataclasses import dataclass

from typing import Optional, Dict, List
import pandas as pd
from etl_lib import DataModel, DataContext
from etl_lib.source import Random_DataSource
from pyspark.sql import SparkSession
from datetime import timedelta


class UserInfo(DataModel):
    hobby: List[str]
    last_seen: Optional[int]
    pet_ages: Dict[str, int]



class User(DataModel):
    login: str
    age: int
    active: bool
    user_info: UserInfo



class Address(DataModel):
    street: str
    city: str
    state: str
    zip: str


class Person(DataModel):
    name: str
    age: float
    height: float
    weight: float
    address: Address


def test_models_generate():

    context = DataContext(
        model=Person,
        start="2021-01-01",
        end="2021-01-02",
        intervals=[timedelta(seconds=1)]
    )


    data = context.model.make_one()

    print(data)



def test_schema_from_DataModel():

    context = DataContext(
        model=Person,
        start="2021-01-01",
        end="2021-01-02",
        intervals=[timedelta(seconds=1)]
    )

    spark_schema = Person.get_schema("spark")
    print(spark_schema)
    avro_schema = Person.get_schema("avro")
    print(avro_schema)

# def generate_records(model: DataModel):
#     """
#     Generate records of self
#     """
#     data = []
#     for record in range(100):
#         data.append(model.fake())
#     return pd.DataFrame(data=data)


# def test_generated_schema(model=User):

#     # Use Tinsel to generate the spark schema from the dataclass
#     schema = transform(model)
#     print(schema)

#     #
#     spark = SparkSession.builder.master("local").getOrCreate()

#     # Generate a Pandas DataFrame
#     pdf = generate_records(model=model)

#     # Load the allocated memory chunk using Arrow as a Spark SQL Dataframe (aka table)
#     df = spark.createDataFrame(data=pdf, schema=model.as_spark_schema())

#     # Print the schema
#     df.printSchema()
#     df.show(truncate=False)
#     return True


if __name__ == "__main__":

    # test_generated_schema()



    # import json
    # from typing import Optional

    # from pydantic_spark.base import SparkBase
    # from pydantic_avro.base import AvroBase

    # class TestModel(SparkBase, AvroBase):
    #     key1: str
    #     key2: int
    #     key2: Optional[str]

    # schema_dict: dict = TestModel.spark_schema()
    # print(json.dumps(schema_dict))


    # schema_dict: dict = TestModel.avro_schema()
        # print(json.dumps(schema_dict))
    # from dataclasses import dataclass

    # from polyfactory.factories.pydantic_factory import ModelFactory

    # from pydantic import BaseModel


    # class PersonFactory(ModelFactory[Person]):
    #     __model__ = Person


    # def test_is_person() -> None:
    #     person_instance = PersonFactory.build()
    #     print(person_instance)
    #     assert isinstance(person_instance, Person)
    
    # test_is_person()
    test_models_generate()


    test_schema_from_DataModel()