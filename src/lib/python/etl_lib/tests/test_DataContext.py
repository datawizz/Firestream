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
