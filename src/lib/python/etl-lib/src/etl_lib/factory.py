from enum import unique
from multiprocessing.sharedctypes import Value
from typing import List, Dict, Union, Optional, Type

from pyspark.sql import DataFrame as SparkDataFrame
from datetime import timedelta, datetime

from etl_lib.context import DataContext
from etl_lib.model import DataModel

from etl_lib.services.spark.client import SparkClient

# Supported sources for data
from etl_lib.source import (
    DataSource,
    SparkDataFrame_DataSource,
    Random_DataSource,
    BrownianMotion_DataSource,
    REST_DataSource,
    SparkREST_DataSource,
    Kafka_DataSource,
)

VALID_SOURCES = [
    Random_DataSource,
    BrownianMotion_DataSource,
    REST_DataSource,
    SparkREST_DataSource,
    # Kafka_DataSource,
    SparkDataFrame_DataSource,
]

import typing
import traceback

# Supported sinks for data
from etl_lib.sink import DataSink, Console_DataSink  # , Kafka_DataSink

VALID_SINKS = [
    Console_DataSink,
]


class DataFactory(DataContext):
    """
    The DataFactory makes Data to order.

    It controls recursively building models and parameterizing them.

    The DataFactory will summon Data from Data Sources

    Uses provided model and access method to

    DataFactorys are to DataModels as the RNA is to DNA.
        i.e. it is a state that is used to translate DNA into Protien (something useable).

    Provides required fields which define a universal representation of data

    * * Locations * *

    * If data is at rest it is stored in Kafka encoded in Avro (with it's schema registered to the topic name)
    * If data is being computed it is stored in the memory of a Spark worker node as a Spark Dataframe
    * If data is being transfered it is usually Avro (Spark -> Kafka -> Spark) or text (Vendor -> Kafka)
    * If data is in a vendor's system
         it is encoded per their policy and delivered usually as JSON, XML, TEXT, etc
         it is consistent with the Avro Schema of the referenced model (dataclass)
            for speed this fact is not always checked.


    ### Fields ###

    model:
        An instance of the DataModel which should be built by the Factory.

    cache:
        If cached results should be used.

    source:
        May be set at DataSource (or subclass) OR a DataContext OR another DataFacory?

    sink:
        The destination to write data to.
        Defaults to StandardOut

    """

    def __init__(
        self,
        start: Union[str, datetime],
        end: Union[str, datetime],
        model: Union[DataModel, None] = None,
        source: Union[Type[DataSource], List[Type[DataContext]]] = None,
        sink: Union[Type[DataSink], List[Type[DataContext]]] = Console_DataSink,
        intervals: Union[List[timedelta], None] = None,
        tickers: Union[List[str], None] = None,
        cache: bool = True,
    ):

        # self.spark_client = SparkClient()

        # super().__init__(
        #     model=model | DataModel,
        #     start=start,
        #     end=end,
        #     cache=cache,
        #     intervals=intervals,
        #     # spark_client=self.spark_client,
        # )
        print("init")
        self.source = source
        self.sink = sink
        self.start = start
        self.end = end
        self.intervals = intervals
        self.tickers = tickers
        self.cache = cache
        self.model = model
        # assert self.source in VALID_SOURCES
        # assert self.sink in VALID_SINKS
        self.spark_client = SparkClient(app_name="TODO_FIX_ME")
        self.built_contexts = {}

    def make(
        self,
        model: Type[DataModel],
        source: Union[Type[DataSource], List[Type[DataSource]]],
        start: Union[str, datetime, None] = None,
        end: Union[str, datetime, None] = None,
        intervals: Union[List[timedelta], None] = None,
        tickers: Union[List[str], None] = None,
    ) -> DataSource:

        """
        Build the provided model using the provided parameters.

        If a optional parameter is not included substitute the parameter from self.

        If the provided model has already been built using the same parameters
        (including sources!) then return it rather than rebuiling it.

        If the provided source is a subclass of DataSource then directly build and return it.

        If the provided source is a list of DataContexts use the SparkDataFrame_DataSource
        to match the built models with the expected inputs.

        Raise an error if there is ambiguity in the requested build.

        Return a DataContext with the built model and a reference to the chain used to build it.
        """

        start = start or self.start
        end = end or self.end
        intervals = intervals or self.intervals
        tickers = tickers or self.tickers

        uniqueness = str([model, source, start, end, intervals, tickers])
        print(uniqueness)
        _uuid = hash(uniqueness)

        # If the exact model has already been built then return the built DataSource
        if _uuid in self.built_contexts.keys():
            return self.built_contexts.get(_uuid)

        else:

            if isinstance(source, list):
                for _source in source:
                    if not isinstance(_source, DataSource):
                        raise ValueError(
                            f"Expected data context but got {type(_source)}"
                        )

                    if not _source.spark_df:
                        raise ValueError(f"Expected dataframe to have been built")

                # Try to construct the model from the provided sources
                # and spark dataframes
                _source = SparkDataFrame_DataSource(
                    sources=source,
                    model=model,
                    start=start,
                    end=end,
                    intervals=intervals,
                    tickers=tickers,
                    spark_client=self.spark_client,
                )
                _source = _source.make()
                self.built_contexts.update({_uuid: _source})
                # _source_list = [x for x in self.built_contexts.values()]
                # return _source_list

            # If the provided source is a DataSource use that directly
            elif issubclass(source, DataSource):
                _source = source(
                    model=model,
                    start=start,
                    end=end,
                    intervals=intervals,
                    tickers=tickers,
                    spark_client=self.spark_client,
                )
                _source.make()
                self.built_contexts.update({_uuid: _source})
                return _source

            else:
                raise ValueError(f"Invalid input")

        return _source

    def show(self):
        """
        Print details about the built models in self
        """

        for k, v in self.built_contexts.items():

            print(k)
            print(v)
            v.spark_df.show()
            v.spark_df.printSchema()

    def run(self, sink: DataSink):
        """
        The command to run the built models
        """
        return True

    def __enter__(self, **kwargs):
        """
        Implements the startup routine when the factory is specified as a context.
        """

        return self

    def __exit__(self, exc_type, exc_value, tb):
        if exc_type is not None:
            traceback.print_exception(exc_type, exc_value, tb)
            # return False # uncomment to pass exception through

        return True
