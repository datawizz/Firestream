from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Any, List, Optional, Union

import pandas as pd
import pyspark.sql.types as t
import pytz
from pandas.tseries.holiday import USFederalHolidayCalendar
from pandas.tseries.offsets import CustomBusinessDay
from pyspark.sql import DataFrame as SparkDataFrame

from etl_lib._utils.git_revision import get_git_revision_hash
from etl_lib.model import DataModel
from etl_lib.services.spark.client import SparkClient


def datetime_to_epoch_nanosecond(dt: datetime) -> int:
    """Format datetime objects to an integer Unix epoch in nanoseconds."""
    return int(pd.to_datetime(dt, utc=True).timestamp() * 10**9)


def datetime_to_epoch_microsecond(dt: datetime) -> int:
    """Format datetime objects to an integer Unix epoch in microseconds."""
    return int(pd.to_datetime(dt, utc=True).timestamp() * 10**6)


def timedelta_to_nanoseconds(td: timedelta) -> int:
    """Convert a Python timedelta to integer nanoseconds."""
    return int(td.total_seconds() * 10**9)


def US_business_days(start: datetime, end: datetime) -> List[pd.Timestamp]:
    """Return the list of US business days between ``start`` and ``end`` (inclusive)."""
    freq = CustomBusinessDay(calendar=USFederalHolidayCalendar())
    return pd.date_range(start=start, end=end, freq=freq).tolist()


@dataclass
class DataIndex(DataModel):
    """A minimal object describing a single time-window partition.

    A DataIndex spans ``(end - start) / interval`` windows in unit time of
    ``interval``. It is used to collate all partitions over a single absolute
    time clock so that streaming actions are bounded and can be checked off
    against an in-memory join dataframe.
    """

    start: datetime
    end: datetime
    offset: int

    @classmethod
    def make(cls, context: "DataContext", interval: timedelta) -> SparkDataFrame:
        """Build a SparkDataFrame of index windows for ``interval``.

        The Spark-side window construction is not yet implemented; the prior
        scaffolding referenced undefined helpers and produced no correct output.
        """
        raise NotImplementedError("DataIndex.make is not yet implemented")


@dataclass
class DataHeader(DataModel):
    """Demarks the beginning boundary of a stream of data.

    The DataHeader includes a start/end timestamp used to partition data into
    intervals based on the timestamp element of the data. Data is processed by
    event time (the wall-clock time of the source system), so processing time
    is some point in the future, potentially well after the event time. Late
    data is managed by the DataFooter.
    """

    start_timestamp: datetime
    end_timestamp: datetime
    start_epoch: int
    end_epoch: int
    interval: int
    uuid: str
    git_commit_hash: str

    @classmethod
    def valid_sources(cls) -> List[type]:
        return [cls]

    @classmethod
    def make(
        cls,
        context: "DataContext",
        date: datetime,
        start: datetime,
        end: datetime,
        interval: timedelta,
    ) -> SparkDataFrame:
        """Build a SparkDataFrame describing this header.

        Not yet implemented — the prior scaffolding referenced an undefined
        ``context`` symbol and could not produce a usable DataFrame.
        """
        raise NotImplementedError("DataHeader.make is not yet implemented")


@dataclass
class DataFooter(DataModel):
    """Demarks the ending boundary of expected data in a stream of event-time data.

    The DataFooter is emitted on the first row whose event time is greater than
    the end timestamp of the most recent DataHeader. A footer advances the
    current DataHeader by 1. The footer's ``block_uuid`` is a hash of the
    header UUID and the row-wise/column-wise hash of the data between them,
    chaining headers and footers like blocks in a block chain.
    """

    header_uuid: str
    data_hash: str
    block_uuid: str

    @classmethod
    def make(cls, context: "DataContext") -> SparkDataFrame:
        raise NotImplementedError("DataFooter.make is not yet implemented")


class DataContext:
    """Holds all parameters describing the how/where/when of accessing data.

    Wraps a DataFrame that is localized with a model schema, time window,
    sampling intervals, and Spark configuration. The DataContext is the unit
    that is passed between DataSources and DataSinks.
    """

    def __init__(
        self,
        model: Optional[type] = None,
        start: Union[str, datetime, None] = None,
        end: Union[str, datetime, None] = None,
        intervals: Optional[List[timedelta]] = None,
        tickers: Optional[List[str]] = None,
        spark_df: Optional[SparkDataFrame] = None,
        spark_client: Optional[SparkClient] = None,
        local_cache: bool = True,
        streaming: Optional[bool] = None,
        period: timedelta = timedelta(days=1),
        sources: Any = None,
    ) -> None:
        self.model = model
        self.start = pd.to_datetime(start, utc=True).astimezone(pytz.utc) if start else None
        self.end = pd.to_datetime(end, utc=True).astimezone(pytz.utc) if end else None
        self.intervals = intervals
        self.tickers = tickers
        self.spark_df = spark_df
        self.spark_client = spark_client
        self.local_cache = local_cache
        self.streaming = streaming if streaming is not None else self._is_streaming()
        self.period = period
        self.cache = local_cache
        self.sources = sources

        if self.cache and self.model is not None and self.start and self.end:
            self.local_path = self._build_local_path()

    def _build_local_path(self) -> str:
        """Build a deterministic local cache path keyed on model + time window."""
        return (
            f"/workspace/.data/{self.model.__name__}"
            f"_start_{self.start.strftime('%Y%m%dT%H%M%S')}"
            f"_end_{self.end.strftime('%Y%m%dT%H%M%S')}"
        )

    def _is_streaming(self) -> bool:
        return self.end is None

    def __len__(self) -> int:
        raise NotImplementedError("DataContext.__len__ is not yet implemented")

    def pdf_from_json(self, data: List[dict], validate: bool = True) -> pd.DataFrame:
        """Parse a list of JSON records into a pandas DataFrame.

        When ``validate`` is True each record is instantiated against the
        context's model so that the schema is enforced.
        """
        if not validate:
            return pd.DataFrame(data=data)
        rows = [self.model.from_dict(record) for record in data]
        return pd.DataFrame(data=rows)

    def pdf_to_json(self, pdf: pd.DataFrame) -> str:
        """Encode a pandas DataFrame as newline-delimited JSON."""
        return pdf.to_json(orient="records", lines=True)

    def pdf_to_sdf(self, pdf: pd.DataFrame, schema: Optional[t.StructType] = None) -> SparkDataFrame:
        """Wrap ``createDataFrame`` for ergonomic pandas→Spark conversion."""
        if not hasattr(self, "spark_client") or self.spark_client is None:
            self.spark_client = SparkClient()
        if schema is None:
            schema = self.model.as_spark_schema()
        return self.spark_client.spark_session.createDataFrame(data=pdf, schema=schema)
