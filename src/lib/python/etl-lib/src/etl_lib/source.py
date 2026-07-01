import abc
import json
import os
import time
from typing import Dict, Generator, List, Optional

import numpy as np
import pandas as pd
from pyspark.sql import DataFrame as SparkDataFrame
from requests import Session

from etl_lib.context import DataContext
from etl_lib.services.kafka.client import KafkaClient


class DataSource(DataContext, metaclass=abc.ABCMeta):
    """Abstract base class from which all data is sourced."""

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)

    @abc.abstractmethod
    def make(self) -> DataContext:
        """Populate ``self.spark_df`` with a Spark DataFrame instance of the model."""
        ...


class SparkDataFrame_DataSource(DataSource):
    """Build a DataModel from a collection of already-built Spark DataFrames.

    Input DataFrames are matched against the target model's ``valid_sources``
    declaration. The composition path is not yet implemented end-to-end.
    """

    def __init__(self, sources: List[DataSource], **kwargs) -> None:
        super().__init__(**kwargs)
        self.sources = self.validate_sources(sources)

    def validate_sources(self, sources: List[DataSource]) -> Dict[str, SparkDataFrame]:
        """Match provided DataFrames against the model's expected sources.

        Raises ``ValueError`` if any provided source does not satisfy one of the
        model's valid_sources entries.
        """
        expected_sources = self.model.valid_sources()
        built: Dict[str, SparkDataFrame] = {}
        for key, options in expected_sources.items():
            for expected_model in options:
                for provided in sources:
                    if type(expected_model) is type(provided.model) and key not in built:
                        built[key] = provided.spark_df
        return built

    def make(self) -> "SparkDataFrame_DataSource":
        raise NotImplementedError("SparkDataFrame_DataSource.make is not yet implemented")


class Random_DataSource(DataSource):
    """Generate random records matching the configured model's schema."""

    def __init__(self, seed: int = 42, **kwargs) -> None:
        super().__init__(**kwargs)
        self.seed = seed
        np.random.seed(self.seed)
        interval = min(self.intervals)
        ms_period = interval.total_seconds() * 10**9
        self.length = int((self.end.value - self.start.value) / ms_period)

    def random_data(self) -> List[dict]:
        """Generate ``self.length`` random records that fit the model schema.

        Depends on ``DataModel.make_one`` which is not yet implemented.
        """
        raise NotImplementedError(
            "Random_DataSource.random_data depends on DataModel.make_one, "
            "which is not yet implemented"
        )

    def random_pdf(self) -> pd.DataFrame:
        return pd.DataFrame.from_records(self.random_data())

    def make(self) -> SparkDataFrame:
        return self.pdf_to_sdf(pdf=self.random_pdf())


class BrownianMotion_DataSource(Random_DataSource):
    """Generate a Geometric Brownian Motion timeseries for stochastic fields."""

    def generate_timestamps(self) -> pd.Series:
        """Generate microsecond timestamps with a small random sub-interval offset."""
        interval = min(self.intervals)
        ms_period = interval.total_seconds() * 10**9
        periods = int((self.end.value - self.start.value) / ms_period)
        arr = np.zeros(shape=(periods, 1), dtype=np.float64)
        arr[0] = self.start.value
        for i in range(1, periods):
            arr[i] = arr[i - 1] + ms_period
        for i in range(1, periods):
            rand = np.random.random_sample(1)
            arr[i] = arr[i] - (rand[0] * ms_period)
        return pd.Series([int(x[0] / 10**3) for x in arr])

    def generate_gbm(
        self,
        mu: float = 0.1,
        sigma: float = 0.10,
        p0: Optional[float] = 10,
    ) -> pd.Series:
        """Generate a Geometric Brownian Motion price series of length ``self.length``.

        ``p0`` seeds the walk; if None, a standard normal sample is used. See
        https://stackoverflow.com/a/45036114/8141780.
        """
        arr = np.zeros(shape=(self.length, 1), dtype=np.float64)
        arr[0] = p0 if p0 is not None else np.random.standard_normal(1)
        for i in range(1, self.length):
            rand = np.random.standard_normal(1)
            arr[i] = arr[i - 1] * np.exp(
                (mu - 0.5 * sigma**2) * (1 / self.length)
                + sigma * np.sqrt(1 / self.length) * rand
            )
        return pd.Series([x[0] for x in arr])

    def make(self, _schema: Optional[dict] = None) -> "BrownianMotion_DataSource":
        """Build a SparkDataFrame replacing stochastic-tagged fields with GBM data.

        Depends on ``Random_DataSource.random_pdf`` (and therefore
        ``DataModel.make_one``), which is not yet implemented.
        """
        raise NotImplementedError(
            "BrownianMotion_DataSource.make depends on Random_DataSource.random_pdf, "
            "which is not yet implemented"
        )


class REST_DataSource(DataSource):
    """Source data backed by a REST API with optional local file caching.

    ``endpoint_urls`` must be set on the instance (typically a pandas DataFrame
    with at least a ``url`` column) before ``cache_rest`` or ``create_sdf`` is
    called.
    """

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)
        self.endpoint_urls: Optional[pd.DataFrame] = None
        self.__http_session: Optional[Session] = None

    def get_rest(
        self,
        url: str,
        params: Optional[dict] = None,
        sleep: int = 0,
        root_node: str = "",
    ) -> List[dict]:
        """GET ``url`` with exponential backoff on transient failures.

        Reuses a single ``requests.Session`` across calls so cookies and
        connection pools persist. Returns the parsed JSON body, or the value at
        ``root_node`` when supplied.
        """
        if self.__http_session is None:
            self.__http_session = Session()
        response = self.__http_session.get(url=url, params=params)

        if response.status_code == 200:
            results = response.json()
            return results[root_node] if root_node else results

        if response.status_code == 404:
            return []

        sleep += 1
        if sleep > 5:
            raise Exception(f"Endpoint never returned after {sleep} attempts: {url}")
        time.sleep(2**sleep)
        return self.get_rest(url=url, params=params, sleep=sleep, root_node=root_node)

    def endpoint_generator(self, endpoint: dict) -> Generator[dict, None, None]:
        """Yield records from ``endpoint['url']`` and enrich each with the other endpoint fields."""
        url = endpoint.pop("url")
        root_node = endpoint.pop("root_node", "")
        for record in self.get_rest(url=url, root_node=root_node):
            if endpoint and record:
                record.update(endpoint)
            yield record

    def cache_rest(self) -> List[str]:
        """Materialize each endpoint to a local JSON file, reusing the cache when present."""
        existing_files = [
            os.path.join(self.local_path, f) for f in os.listdir(self.local_path)
        ]
        files = []
        for _, row in self.endpoint_urls.iterrows():
            endpoint = row.to_dict()
            url = row["url"]
            filename = "".join(x if x.isalnum() else "_" for x in url) + ".json"
            full_path = os.path.join(self.local_path, filename)
            if full_path not in existing_files:
                with open(full_path, "w") as f:
                    for record in self.endpoint_generator(endpoint=endpoint):
                        f.write(json.dumps(record) + " \n")
            files.append(full_path)
        return files

    def create_sdf(self) -> SparkDataFrame:
        """Load endpoint data into a SparkDataFrame using the model's Spark schema."""
        schema = self.model.as_spark_schema()
        if self.local_cache:
            files = self.cache_rest()
            if not self.streaming:
                return self.spark_client.spark_session.read.schema(schema).json(files)
            return self.spark_client.spark_session.readStream.schema(schema).json(self.local_path)

        data = self.pdf_from_json(data=list(self.endpoint_generator({})))
        return self.spark_client.spark_session.createDataFrame(data=data, schema=schema)

    def make(self) -> "REST_DataSource":
        self.spark_df = self.create_sdf()
        return self


class SparkREST_DataSource(REST_DataSource):
    """Distribute REST calls across Spark workers via a UDF.

    The UDF-based distributed REST path was never completed (the original
    implementation referenced undefined helpers and could not be executed).
    """

    def make(self) -> SparkDataFrame:
        raise NotImplementedError("SparkREST_DataSource.make is not yet implemented")


class Kafka_DataSource(DataSource):
    """Stream or batch-read a Kafka topic into a Spark DataFrame.

    Construction wires up the Kafka client and the topic name; the actual
    Avro-deserialization path depends on ABRiS-side helpers that are not yet
    integrated, so ``make`` and ``df_from_kafka`` raise NotImplementedError.
    """

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)
        self.kafka_client = KafkaClient()
        self.topic = self.model.__name__
        self.schema_registry_config = {
            "schema.registry.url": self.kafka_client.schema_registry_dict.get("url"),
        }

    def make(self) -> "Kafka_DataSource":
        raise NotImplementedError("Kafka_DataSource.make is not yet implemented")

    def df_from_kafka(self, topic: str, streaming: bool = False) -> SparkDataFrame:
        """Read ``topic`` as a Spark DataFrame, decoding the Avro payload.

        Avro decoding relies on ABRiS JVM helpers that are not yet bridged.
        """
        raise NotImplementedError("Kafka_DataSource.df_from_kafka is not yet implemented")


class SparkRateMicroBatch_StreamingSource(DataSource):
    """Streaming source backed by Spark's rate-micro-batch source.

    The micro-batch loop was never finished — left as a NotImplementedError
    until the streaming pipeline contract is defined.
    """

    def make(self) -> "SparkRateMicroBatch_StreamingSource":
        raise NotImplementedError("SparkRateMicroBatch_StreamingSource.make is not yet implemented")
