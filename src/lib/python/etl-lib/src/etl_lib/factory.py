import traceback
from datetime import datetime, timedelta
from typing import List, Type, Union

from etl_lib.context import DataContext
from etl_lib.model import DataModel
from etl_lib.services.spark.client import SparkClient
from etl_lib.sink import Console_DataSink, DataSink
from etl_lib.source import (
    BrownianMotion_DataSource,
    DataSource,
    Kafka_DataSource,
    REST_DataSource,
    Random_DataSource,
    SparkDataFrame_DataSource,
    SparkREST_DataSource,
)

VALID_SOURCES = [
    Random_DataSource,
    BrownianMotion_DataSource,
    REST_DataSource,
    SparkREST_DataSource,
    Kafka_DataSource,
    SparkDataFrame_DataSource,
]

VALID_SINKS = [
    Console_DataSink,
]


class DataFactory(DataContext):
    """Build DataModels by composing DataSources.

    A factory holds shared parameters (time window, intervals, tickers) and a
    cache of already-built models, so a model referenced multiple times in a
    composition graph is only constructed once.
    """

    def __init__(
        self,
        start: Union[str, datetime],
        end: Union[str, datetime],
        model: Union[DataModel, None] = None,
        source: Union[Type[DataSource], List[Type[DataContext]], None] = None,
        sink: Union[Type[DataSink], List[Type[DataContext]]] = Console_DataSink,
        intervals: Union[List[timedelta], None] = None,
        tickers: Union[List[str], None] = None,
        cache: bool = True,
        app_name: str = "etl-lib",
    ):
        self.source = source
        self.sink = sink
        self.start = start
        self.end = end
        self.intervals = intervals
        self.tickers = tickers
        self.cache = cache
        self.model = model
        self.spark_client = SparkClient(app_name=app_name)
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
        """Build ``model`` from ``source``, reusing cached results when possible.

        ``source`` may be a single ``DataSource`` subclass — in which case it is
        instantiated and ``.make()`` called directly — or a list of already-built
        ``DataSource`` instances composed via ``SparkDataFrame_DataSource``.
        """
        start = start or self.start
        end = end or self.end
        intervals = intervals or self.intervals
        tickers = tickers or self.tickers

        cache_key = hash(str([model, source, start, end, intervals, tickers]))
        if cache_key in self.built_contexts:
            return self.built_contexts[cache_key]

        if isinstance(source, list):
            for _source in source:
                if not isinstance(_source, DataSource):
                    raise ValueError(f"Expected DataSource but got {type(_source)}")
                if not _source.spark_df:
                    raise ValueError("Expected source DataFrame to have been built")
            built = SparkDataFrame_DataSource(
                sources=source,
                model=model,
                start=start,
                end=end,
                intervals=intervals,
                tickers=tickers,
                spark_client=self.spark_client,
            ).make()
        elif isinstance(source, type) and issubclass(source, DataSource):
            built = source(
                model=model,
                start=start,
                end=end,
                intervals=intervals,
                tickers=tickers,
                spark_client=self.spark_client,
            )
            built.make()
        else:
            raise ValueError(f"Invalid source: {source!r}")

        self.built_contexts[cache_key] = built
        return built

    def show(self) -> None:
        """Print every model built by this factory along with its schema."""
        for key, source in self.built_contexts.items():
            print(key)
            print(source)
            source.spark_df.show()
            source.spark_df.printSchema()

    def run(self, sink: DataSink) -> bool:
        """Execute the built model graph and write to ``sink``.

        The end-to-end execution pipeline is not yet implemented.
        """
        raise NotImplementedError("DataFactory.run is not yet implemented")

    def __enter__(self) -> "DataFactory":
        return self

    def __exit__(self, exc_type, exc_value, tb) -> bool:
        if exc_type is not None:
            traceback.print_exception(exc_type, exc_value, tb)
        return True
