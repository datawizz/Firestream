from datetime import datetime, timedelta, date
import os
import json
import pandas as pd
import numpy as np
from typing import Union, List
from pyspark.sql import DataFrame as SparkDataFrame
from etl_lib.model import DataModel

# from etl_lib.types import (
#     TimeDelta_Type,
#     Timestamp_Type,
#     Float_Type,
#     TimeDelta_Type,
#     Date_Type,
# )
from etl_lib.services.spark.client import SparkClient
from etl_lib.services.kafka.client import KafkaClient


import pytz
import pyspark.sql.functions as F
from typing import Dict
import copy

from pandas.tseries.offsets import CustomBusinessDay
from pandas.tseries.holiday import USFederalHolidayCalendar

from typing import Optional, Iterator
from enum import Enum

from pyspark.sql.window import Window
from typing import Type
import pyspark.sql.types as t
from etl_lib.utils.git_revision import get_git_revision_hash
from dataclasses import dataclass


def datetime_to_epoch_nanosecond(dt: datetime):
    """Format datetime objects to a integer Unix epoch in nanoseconds"""
    return int(pd.to_datetime(dt, utc=True).timestamp() * 10**9)


def datetime_to_epoch_microsecond(dt: datetime):
    """Format datetime objects to a integer Unix epoch in microseconds"""
    return int(pd.to_datetime(dt, utc=True).timestamp() * 10**6)


def timedelta_to_nanoseconds(td=pd.Timedelta):
    """
    Convert a python timedelta to a integer with units of nanoseconds
    """
    return td.total_seconds()


def US_business_days(start: datetime, end: datetime) -> List[pd.Timestamp]:
    """
    Return a list of business days between the dates provided (inclusive)
    """

    if True:
        # Make use of business days calendar
        _freq = CustomBusinessDay(calendar=USFederalHolidayCalendar())
    else:
        _freq = timedelta(days=1)

    days = pd.date_range(start=start, end=end, freq=_freq).tolist()

    return days


"""
A five step broadcast join

Find the most compact representation of the timespans that can be broadcasted

A DF of business Days between context.start and context.end

For each day, for each interval, create a DataHeader

For each interval define a set of monotonicaly increasing offsets that span the day

    if interval is very small then this is very large
    -> limit to 1 second intervals 86400 paritions


    for each interval that is less than 1 second
        find the offsets within a second in terms of milliseconds -> 1000 rows

broadcast to all nodes
    df_days = <1000 rows
    df_DataHeader = intervals * df_days < 10,000 (for 10 intervals and 1000 days)
    df_offset_seconds = < 86400 (seconds in a 24 hours)
    df_offset_subseconds = < 1000 (milliseconds in a second, the smallest timeinterval that is valid)

    = less than 100,000 rows to be broadcast with 1000 days @ millisecond resolution


join natural data such that
    data.event_time >= Dataheader.start + offset * interval
    data.event_time <= (Dataheader.start + (offset * interval) + interval - 1 (nanosecond) )



join natural data such that:

    for the two dataframes in question:

        1. Join the DF with the DataHeader.UUID
            left join df.event_time BETWEEN dataheader.start and dataheader.end

        2. Join the DF with the offset_seconds
            left join df.seconds_since_midnight = offset_seconds

        3. Join the DF with the offset_subseconds
            left join df.subseconds_since_second = offset_subseconds

    now join the two dataframes on the join keys that were appeneded
    This is a shuffle join and thus has no size limit
        there should be no required shuffle as part of this is already ordered
        data within subsecond offsets is assumed to be ordered.


0. A DF of Days
    limit to business days

1. A DF of DataHeaders
    A DataHeader defines a batch of data.
    There can be headers inside of headers.
    By default each day has it's own header.
    Each interval specified has a DataHeader

2. A DF of offsets as intgers
    Offsets for any DataHeader[interval] are static and don't need to be exploded, they can be broadcast as is 
    must be in terms of min(interval) < 1 day and equally dividiable

3. Some natural data
    contains event time

join them all!
1. Broadcast the days, headers, offsets. Should be smol (<2gb)

3. Join the offsets to the headers (stays on the same worker)
4. Join the natural data to the offset x header df

"""


@dataclass
class DataIndex(DataModel):
    """

    The DataIndex is a minimal object used to partition timeseries.

    Partitions are generated relative to a offset.


    A DataModel that spans the dot product of (start - end) * interval in unit time of interval
    This is used to colate all partitions over a single absolute time clock
    All data exists somewhere in the index
    Since Kafka provides absolute ordering within a single topic, this is meant to be a
    in memory distributed "join" dataframe which ensures that actions are bounded and checked off in streaming live mode

    An instance of this class is a epoch in ML terms.
    Each window is a batch within the epoch.
    The interval naturally provides the minimum frame advance.

    The DataIndex is used by the footer and header for closing a interval.

    interval_offset:
        The offset for the interval of the trading day.


    offsets:
        A monotonically increasing array of integers representing the offset, in terms of intervals,
        from the start_timestamp to the end_timestamp.
        The product of (start - end) / interval


    """

    start: datetime
    end: datetime
    offset: int

    @classmethod
    def make(cls, context, interval: int) -> SparkDataFrame:
        # The Index is built using the context rather than a source

        if True:  # ignore_closed_market:
            # Make use of business days calendar
            _freq = CustomBusinessDay(calendar=USFederalHolidayCalendar())
        else:
            _freq = timedelta(days=1)

        # Create a DataFrame of Dates to start
        days_df = pd.DataFrame(
            data=pd.date_range(start=context.start, periods=1, freq=_freq).to_list(),
            columns=["date"],
        )

        # TODO use market hours
        # Generate over timestamps that respect market hours
        days_df = context.pdf_to_sdf(df=days_df)

        # The number of nanoseconds of offset to each frame
        interval_nanoseconds = interval.total_seconds() * 10**9
        array_length = (context.end - context.start).total_seconds() * 10**9
        start_nanoseconds = 0

        # Create an array of the interval in nanoseconds repeated for every window that
        # exists in the timeframe
        df = days_df.withColumn(
            "offset", F.expr(f"array_repeat({interval_nanoseconds}, {array_length})")
        )

        # Gaurenteed to be in the correct order but not sequential
        # See https://spark.apache.org/docs/latest/api/sql/index.html#monotonically_increasing_id
        df = df.withColumn("index", F.expr("SELECT monotonically_increasing_id()"))

        # Define a window over each date/epoch and the increasing id
        window_spec = Window.partitionBy("date").orderBy("index")

        # Switch to a sequential index of integers
        df = df.withColumn("index", F.row_number().over(window_spec))

        # Apply the offset to each row as a vector
        # Where the product of the index and the offset is the total offset.
        df = df.withColumn(
            F.col("window_end"),
            F.lit(start_nanoseconds) + (F.col("offset") * F.col("index")),
        )

        # Start windows on the first nanosecond in the frame
        df = df.withColumn(
            F.col("window_start"), F.col("window_end") - F.col("offset") + F.lit(1)
        )

        df = df.select("index", "window_start", "window_end")
        df = df.withColumn(
            "uuid",
            F.sha2(F.concat_ws("", [F.col(x).cast("string") for x in df.columns])),
        )

        return df


@dataclass
class DataHeader(DataModel):
    """
    The DataHeader demarks the beginning boundary of a stream of data.

    The DataHeader includes a start and end timestamp which is used to
    partition data into intervals based on the timestamp element of the data.

    Data is processed based on Event Time, aka the wall clock time that the event
    happened in the source system. This means that the processing time is some point in the future
    potentially well after the event time.

    Late data is managed by the DataFooter.

    ** Arguments **

    date:
        The date as a Timezoned Timestamp at 12:00am local time
    start:
        A timezoned timestamp to start a sequence of data
    end:
        A timezoned timestamp to end a sequence of data
    interval:
        A timedelta indicating the period that the data should be split into

    ** Fields **

    start_timestamp:
        A timestamp type of the starting window.
        Used in Spark since microseconds are the smallest unit of time allowed.

    start_epoch:
        The start_timestamp in Unix Epoch nanoseconds format.
        Useful for stream-stream joins.

    end_timestamp:
        A timestamp of the last point in time to accept new data without considering it late.

    end_epoch:
        The end_timestamp in Unix Epoch nanoseconds format.

    interval:
        A integer which represents the number of nanoseconds between each "window" of data between start and end.
        Every timeseries can be divided into windows which demark boundaries of temporally similar data.
        i.e. to create a 15 minute Candle this value would be 15*60*(10^9)

    # offset_seconds:
    #     An array of monotonically increasing integers representing the offset from the start_epoch
    #     in units of seconds

    # offset_subseconds:
    #     An array of monotonically increasing integers representing the offset from each second
    #     in terms of fractions of a second.
    #     Min 10**9 = 1 nanosecond



    uuid:
        Uniquely identifies a specfic DataHeader record.
        Referenced by the DataFooter to create a complete "Block"

    git_commit_hash:
        The commit ID of the code that was used to process the data in this DataHeader.

    """

    start_timestamp: datetime
    end_timestamp: datetime
    start_epoch: int
    end_epoch: int
    interval: int  # BigInt
    uuid: str
    git_commit_hash: str

    @classmethod
    def valid_sources(cls):
        """ """
        return [cls]

    @staticmethod
    def make(
        date: datetime, start: datetime, end: datetime, interval: timedelta
    ) -> SparkDataFrame:
        """
        Create an instance of self as a SparkDataFrame
        """

        _data = []

        # Spark accepts microseconds for time based windows
        # But support for nanosecond records between the windows is implemented

        start_epoch = datetime_to_epoch_nanosecond(dt=start)
        end_epoch = datetime_to_epoch_nanosecond(dt=end)
        epoch_delta = end_epoch - start_epoch

        manual_schema = t.StructType(
            [
                t.StructField("start_epoch", t.LongType(), False),
                t.StructField("end_epoch", t.LongType(), False),
                t.StructField("interval", t.DoubleType(), False),
                t.StructField("git_commit_hash", t.StringType(), False),
            ]
        )

        git_hash = get_git_revision_hash()

        _interval = timedelta_to_nanoseconds(td=interval)

        if _interval > epoch_delta:
            raise ValueError(
                f"The provided interval:{interval} is larger than the provided provided time window start:{context.start} end:{context.end}"
            )
        if not epoch_delta % _interval == 0:
            raise ValueError(
                f"The provided interval:{interval} does not evenly divide the provided time window start:{context.start} end:{context.end}"
            )

        # Find the number of seconds in each period
        # This is at most 24 hours and therefore 86400 seconds
        num_one_second_period = int((epoch_delta / _interval) / 10**9)

        # Find the number of fractions of a second that the interval subdivides a second into
        if interval < timedelta(seconds=1):
            fractions_of_second = timedelta(seconds=1) / interval

            # Fix the fractions of a second to millisecond precision
            # TODO find a way to remove this limit which was placed to keep the size of the data from becoming unmanagable
            if fractions_of_second >= 1000:
                fractions_of_second = 1000

        _data = {
            "start": start_epoch,
            "end": end_epoch,
            "interval": _interval,
            "fractions_of_second": fractions_of_second,
            "git_commit_hash": git_hash,
        }

        df = pd.DataFrame(_data)
        df = context.pdf_to_sdf(pdf=df, schema=manual_schema)

        df = df.withColumn(
            "uuid",
            F.sha2(
                F.concat("start", "end", "interval"),
                256,
            ),
        )

        # TODO this
        df = df.withColumn("start_timestamp", F.to_timestamp(F.col("start") / 10**9))
        df = df.withColumn("end_timestamp", F.to_timestamp(F.col("end") / 10**9))

        # Make this available on all workers
        F.broadcast(df=df)
        return df


@dataclass
class DataFooter(DataModel):
    """
    A DataFooter demarks the ending boundary of expected data in a stream of event time data.


    The DataFooter is emitted upon the first Row received when has a time greater than the
    end timestamp of the most recent DataHeader.

    A footer advances the current DataHeader by 1.

    header_uuid:
        The UUID of the header that this footer represents the conclusion of
    data_hash:
        A row wise then column wise hash of all data between this footer and the next header
    block_uuid:
        A hash of the data_hash and the header_uuid.
        Therefore a DataFooter acts as a block in a block chain!
        Each Footer can be referenced in context to the header to indipendently validation
        the uniqueness

    """

    header_uuid: str
    data_hash: str
    block_uuid: str

    @classmethod
    def make(cls, context) -> SparkDataFrame:
        df = ""
        return df


@dataclass
class LateDataFooter(DataModel):
    """
    Late Data is any data whose event time is greater than the current reading frame.

    A Late DataFooter is meant as a appendix to the DataFooter already emitted.

    Monitoring the difference between the footer and the late footer shows the
    total number of an delta time of late data
    """

    pass


class BookEnds(DataModel):
    """
    A model which is the union of the different types of meta data

    This defines a "block" which can be hashed and chained

    """

    event: Union[DataFooter, DataHeader, LateDataFooter]




class DataContext:

    """
    The Data Context abstracts a DataFrame which is localized with some parameters.
    The aim is to hold all configurations regarding the how, where, and when of accessing data.
    The DataContext contains a distributed index that contains metadata about the built model.


    * * Arguments * *

    model:
        The statically typed schema of the data that this context will make available

    start:
        The earliest time that data exists for the specified model

    end:
        If the end is not specified assume data is still being produced in the Kafka topic

    spark_client:
        Used to interact with Spark
        If not provided on the construct call of this object then it is created

    spark_df:
        A Spark Dataframe that is populated by the "make()" method of the model to create a instance of self

    cache:
        Control if the data that is downloaded from external sources should be cached locally.
        Defaults to True.

    local_path:
        A generated unique path for data that is a instance of the model.
        Only used if cache=True

    streaming:
        Controls if the methods called to construct the Spark DataFrame should treat it as a streaming DataFrame
        Defaults to False #TODO streaming should be the default state for development.

    intervals:
        A list of timedetla(s) which indicate the frequency of sampling the data
        Each row in spark_df is associated with exactly one interval.

    period:
        timdelta
        The amount of time between each "batch" of data
        defaults to 1 day

    use_business_days:
        default True
        If the date array created in start and end should have US Holidays and Weekends filtered out

    """

    def __init__(self, **kwargs) -> None:

        self.model = kwargs.get("model")
        self.start = pd.to_datetime(kwargs.get("start"), utc=True).astimezone(pytz.utc)
        self.end = pd.to_datetime(kwargs.get("end"), utc=True).astimezone(pytz.utc)

        self.intervals = kwargs.get("intervals")
        self.tickers = kwargs.get("tickers", None)
        self.spark_df = kwargs.get("spark_df", None)
        self.spark_client = kwargs.pop("spark_client", None)
        self.local_cache = kwargs.get("local_cache", True)
        self.streaming = kwargs.get("streaming", self.__is_streaming())

        self.period = kwargs.get("period", timedelta(days=1))

        # self.index_df = kwargs.get(
        #     "index", self.create_index()
        # )  # TODO could be a key:value store like Redis
        # # TODO this should be moved to the DataModel, that way the model provides it own index, and is sensative to field perculiarities

        self.cache = kwargs.get("local_cache", True)

        if self.cache:
            self.local_path = self.__build_local_path()

        self.sources = kwargs.get("sources", None)

        # self.source = kwargs.get("source", None)
        # self.sink = kwargs.get("sink", None)

    def __build_local_path(self):
        """
        Use a consistent naming convention to build the local cache path
        Assume that objects are overritten by continued calls
        """
        #TODO update this to form a path that is universal on S3
        path = f"/workspace/.data/{self.model.__name__}_start_{self.start.strftime('%Y%m%dT%H%M%S')}_end_{self.end.strftime('%Y%m%dT%H%M%S')}"

        # # Make the directory if it does not exist
        # if not os.path.exists(path):
        #     os.mkdir(path)
        return path

    def _set_mode(self, mode: Optional[str]):
        if mode in VALID_MODES:
            return mode
        else:
            raise ValueError(
                f"{mode} was provided but only {str(VALID_MODES)} are allowed"
            )

    def __is_streaming(self):
        if not self.end:
            return True
        else:
            return False

    def __len__(self):
        #TODO this should be the number of rows in the DataFrame
        return 1
    


    def paritioner():
        #TODO
        """
        A partitioner is a function that takes a row and returns a partition key
        Passed a DataFrame this will return a DataFrame with a new column "partition_key"

        For speed the partitioner is implemented in Scala and then wrapped in Python as a Spark UDF

        
        Spec:
            The partitioner should be a function that takes a row and returns a string
            The string should be a valid partition key for the underlying storage system (S3, HDFS, parquet)
            The partition key should be a valid path is a POSIX path
            The partition key is composed a partial set of fields of the row / DataFrame which are passed as arguments to the partitioner
            The partition key is a tuple of a short key and a long key.
            The short key is composed of several parts:
                1. A kafka_key which is used in Kafka to guarantee ordering
                2. UUID a globally unique identifier for that record which may or may not be the same as the Kafka key
                3. The event_time which is the time that the event was timestamped in the source system
            
            The paritioner is a object which has several methods:
                1. A method to create a partition key from a row
                2. A method to create a partition key from a DataFrame
                3. A method to create a partition key for POSIX path on S3 which is based on the event time
                4. A method to create a partition key for kafka
                    this include configuring the kafka timestamp to use the event time
            

        """

        def __init__(self, partitioner: Callable[[Row], str]):
            self.partitioner = partitioner

        def __call__(self, row: Row) -> str:
            return self.partitioner(row=row)

    # def create_index(self) -> SparkDataFrame:
    #     """
    #     Create a DataIndex for each provided interval

    #     # TODO what happens when this is a HUGE number? Should be it broadcasted?
    #     """

    #     intervals_sorted = self.intervals.sort()
    #     df = DataIndex.make(context=self, interval=intervals_sorted[0])
    #     for interval in intervals:
    #         higher_interval_df = DataIndex.make(context=self, interval=interval)
    #         df.unionByName(higher_interval_df)

    #     return df

    # def create_index_old(
    #     self, ignore_closed_market=True, include_partition_columns=True
    # ) -> pd.DataFrame:
    #     """
    #     Take the Start, End, and Intervals configured in self to create a index

    #     ## Fields ###

    #     window:
    #         The start and stop timestamp of each interval
    #     offset:
    #         The increment of the interval in nanoseconds
    #     days:
    #         The int offset in terms of days
    #         identifies batches for DataLoader
    #     i:
    #         The numerical offset of the index as a number that increases from 0 each day

    #     ## Returns ##

    #     Return a Pandas multiindex over the defined space
    #     """

    #     if ignore_closed_market:
    #         # Make use of business days calendar
    #         _freq = CustomBusinessDay(calendar=USFederalHolidayCalendar())
    #     else:
    #         _freq = timedelta(days=1)

    #     days = pd.date_range(start=self.start, periods=1, freq=_freq).tolist()
    #     partition_cols = {}  # if not self.model.partition_cols()
    #     windows = []
    #     for interval in self.intervals:
    #         frames = pd.interval_range(start=self.start, end=self.end, freq=interval)
    #         windows.extend(
    #             [
    #                 {
    #                     "start": x.left.timestamp(),
    #                     "end": x.right.timestamp(),
    #                     "interval": interval.total_seconds(),
    #                     **partition_cols,
    #                 }
    #                 for x in frames.to_list()
    #             ]
    #         )
    #     return pd.DataFrame(windows)

    def pdf_from_json(self, data: List[dict], validate=True) -> pd.DataFrame:
        """
        Parse JSON data as a instance of the schema in self.Model
        """
        if validate:
            datars = []
            for record in data:
                # Check the validity of each record by instantiating a class against it
                r = self.model.from_dict(record)
                datars.append(r)
            df = pd.DataFrame(data=datars)
            return df
        else:
            # Don't check the validity of each record
            return pd.DataFrame(data=data)

    def pdf_to_json(self, pdf: pd.DataFrame) -> str:
        """
        Create a json string that is the encoding of the Pandas DataFrame
        # TODO add validation of the schema before creating json?
        """

        return pdf.to_json(orient="records", lines=True)

    def pdf_to_sdf(self, pdf: pd.DataFrame, schema=None) -> SparkDataFrame:
        """
        Wraps spark Pandas interface for convenience
        """
        # Controls if the SparkClient should be created
        # This is not enforced but some code of this class (like the below) cannot be executed on executors in Spark, only drivers
        if not hasattr(self, "spark_client"):
            self.spark_client = SparkClient()

        if not schema:
            schema = self.model.as_spark_schema()

        return self.spark_client.spark_session.createDataFrame(data=pdf, schema=schema)
