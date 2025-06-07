from dataclasses import dataclass, fields

from etl_lib.services.kafka.client import KafkaClient

import pyspark.sql.functions as F

from pyspark.sql.types import (
    StructType,
    StructField,
    IntegerType,
    StringType,
    ArrayType,
    StringType,
)
from typing import Union, Optional


import os
import json
from requests import Session
from pyspark.sql import DataFrame as SparkDataFrame
import pandas as pd

import time
from typing import Union, List, Generator, Iterator, Dict
from enum import Enum
from datetime import datetime, timedelta


from etl_lib import DataModel
from etl_lib.context import DataContext


from etl_lib.services.kafka.client import KafkaClient

import pyspark.sql.functions as F

from typing import Optional
from dataclasses import dataclass
import numpy as np
import pandas as pd

from etl_lib.services.spark.client import SparkClient


# import pyspark.pandas as ps


# # For running Spark
# from pyspark.sql import SparkSession
# spark = SparkSession.builder.appName("Spark").getOrCreate()


# r = ps.timedelta_range(start='1 day', periods=4)

# print(r)

import abc


class DataSource(DataContext, metaclass=abc.ABCMeta):
    """
    The Abstract Base Class from which all data is sourced

    #TODO maybe it is cleaner to only include the index instead of start, end, intervals
    """

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)

        #

    # @abc.abstractmethod
    # def __getitem__(self, index: Union[datetime, int, None] = None) -> Iterator[dict]:
    #     """
    #     Returns an iterator over data starting at the index
    #     TODO this is used in pytorch dataloader
    #     """
    #     ...

    @abc.abstractmethod
    def make(self) -> DataContext:
        """
        Populate self with a Spark DataFrame instance of the model
        """
        ...


class SparkDataFrame_DataSource(DataSource):
    """
    Implements the DataSource and DataModel API over a collection of Spark DataFrames.

    Input DataFrames are examined for compatibility with the valid sources defined in
    the DataModel to be built.

    This abstraction allows for multiple Spark DataFrames to be used as inputs to a
    single model through schema checking.
    """

    def __init__(self, sources: List[DataSource], **kwargs) -> None:

        super().__init__(**kwargs)

        self.sources = self.validate_sources(sources)

    def validate_sources(self, sources: List[DataSource]) -> Dict[str, SparkDataFrame]:
        """
        Compare the list of provided dataframes for compatibility with the defined sources
        of the DataModel. If there is ambiguity raise an error instead of guessing.

        1. Get the valid sources from the model
        2. Compare the provided DataContexts with self
            A valid context should have the same or more granual time resolution (min interval)
            compared to the model to be built
        3. If there are multiple provided contexts which could be used raise an error #TODO handle this better



        A DataModel can include a source that is generic by specifying the
        required fields and types as a dataclass definition inside their own
        "valid source" method.
        """

        expected_sources = self.model.valid_sources()
        _build_sources = {}

        # Iterate over the dictionary of sources required by the model
        for _key, _value in expected_sources.items():
            # Iterate over the options provided that fit the source
            for expected_model in _value:
                # Iterate over the provided sources
                for provided_source in sources:

                    provided_model = provided_source.model

                    # TODO compare the schema instead to allow dynamic composistion.
                    if (
                        type(expected_model) is type(provided_model)
                        and _key not in _build_sources.keys()
                    ):
                        _build_sources.update({_key: provided_source.spark_df})
                    # elif (
                    #     type(expected_model) is type(provided_model)
                    #     and _key in _build_sources.keys()
                    # ):
                    #     raise ValueError("Identical sources provided?")
                    elif not type(expected_model) is type(provided_model):
                        continue
                    else:
                        raise ValueError("got to end of frame")

        return _build_sources

    def make(self) -> DataSource:
        """
        For a DataModel to be built using this source it should accept a dictionary
        of key : SparkDataFrame to it's make method.
        """

        df = self.model.make(context=self.sources)
        self.spark_df = df

        return self


class Random_DataSource(DataSource):
    """
    Extends DataSource to include methods to generate random records for the provided model
    """

    def __init__(self, **kwargs) -> None:

        super().__init__(**kwargs)
        self.seed = 42  # The answer to life, the universe, and everything.
        np.random.seed(self.seed)

        self.length = self.period

        _interval = min(self.intervals)
        # The number of microseconds in each period of the smallest interval
        _ms_period = _interval.total_seconds() * 10**9
        # The total number of periods in the range
        periods = int(((self.end.value - self.start.value)) / _ms_period)
        self.length = periods


        
    def random_data(self) -> SparkDataFrame:
        """
        Populate data that fits with the schema of the provided model

        # TODO Add configs to control the order, size, normality of
        # the distribution based on what the model would expect for these values
        """

        data = []

        for i in range(self.length):
            # data.append({"name": "steve", "address": "123 Front Street"})
            data.append(self.model.make_one())
        return data

    def random_pdf(self) -> pd.DataFrame:
        """
        Populate a DataFrame with sane values given the Types
        Create a Pandas Dataframe populated with random data that is an instance of the provided DataModel
        Because the provided DataModel may have complex field types recurse to populate those as well.
        """
        data = self.random_data()
        return pd.DataFrame.from_records(data)

    def make(self) -> SparkDataFrame:

        """
        Return a Spark Dataframe filled with random data
            where the datum is an instance of DataModel
        """

        data = self.random_pdf()
        return self.pdf_to_sdf(pdf=data)


class BrownianMotion_DataSource(Random_DataSource):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)

    def generate_timestamps(self) -> pd.Series:
        """
        Use a Random variable to find the percent offset of a microsecond timestamp
        between two sampling periods
        """
        # # Micro seconds since epoch
        # ts = self.start.value / 10 ** 3
        # print(ts)

        _interval = min(self.intervals)

        # The number of microseconds in each period of the smallest interval
        _ms_period = _interval.total_seconds() * 10**9
        print(_ms_period)

        # The total number of periods in the range
        periods = int(((self.end.value - self.start.value)) / _ms_period)
        print(periods)

        # Malloc the data in the array
        _array = np.zeros(shape=(periods, 1), dtype=np.float64)

        # The starting value is the configured start
        _array[0] = self.start.value
        print(_array[0])

        for t in range(periods):
            if t == 0:
                continue
            _array[t] = _array[t - 1] + _ms_period

        # Each element in the final array is start + interval - offset
        for t in range(periods):
            if t == 0:
                continue
            rand = np.random.random_sample(1)
            _array[t] = _array[t] - (rand[0] * _ms_period)

        return pd.Series([int(x[0] / 10**3) for x in _array])

    def generate_gbm(
        self,
        mu: Optional[float] = 0.1,
        sigma: Optional[float] = 0.10,
        p0: Optional[float] = 10,
    ) -> pd.Series:
        """

        Generate Data using Geometric Brownian Motion (GBM).

        Index is a random walk of microsecond epoch timestamps


        See https://stackoverflow.com/a/45036114/8141780.

        ## Args ##

        length:
            The number of elements to be generated in the returned array.

        p0:
            The price at time zero (or index 0).
            This is the node from which all prices are generated in the random walk.
            This is only used to get *future* random measurements.
            If not provided the seed is used.

        mu:
            Drift, or mean of the percentage change.

        sigma:
            The standard deviation of the percentage change

        """

        # The array which will be populated
        _array = np.zeros(shape=(self.length, 1), dtype=np.float64)

        if p0:
            # The random walk starts at a specific value
            _array[0] = p0
        else:
            _array[0] = np.random.standard_normal(1)

        for t in range(self.length):
            if t == 0:
                continue
            else:
                rand = np.random.standard_normal(1)
                _array[t] = _array[t - 1] * np.exp(
                    (mu - 0.5 * sigma**2) * (1 / self.length)
                    + sigma * np.sqrt(1 / self.length) * rand
                )
        return pd.Series([x[0] for x in _array])

    # @F.udf(returnType=StringType())
    # def gbm_spark_function(self):
    #     """
    #     Generates a series of GBM over a dataframe
    #     """
    #     pass

    # def generate_symbol(self,
    #                     # symbol: tp.Label,
    #                     # index: tp.Index,
    #                     S0: float = 100.,
    #                     mu: float = 0.,
    #                     sigma: float = 0.05,
    #                     T: tp.Optional[int] = None,
    #                     I: int = 1,
    #                     seed: tp.Optional[int] = None):
    #     """Generate the symbol using `generate_gbm_paths`.

    #     Args:
    #         symbol (str): Symbol.

    #         index (pd.Index): Pandas index.

    #         S0 (float): Value at time 0.

    #             Does not appear as the first value in the output data.
    #         mu (float): Drift, or mean of the percentage change.
    #         sigma (float): Standard deviation of the percentage change.
    #         T (int): Number of time steps.

    #             Defaults to the length of `index`.
    #         I (int): Number of generated paths (columns in our case).
    #         seed (int): Set seed to make the results deterministic.
    #     """

    #     if T is None:
    #         T = len(index)
    #     out = generate_gbm_paths(S0, mu, sigma, T, len(index), I, seed=seed)[1:]
    #     if out.shape[1] == 1:
    #         return pd.Series(out[:, 0], index=index)
    #     columns = pd.RangeIndex(stop=out.shape[1], name='path')
    #     return pd.DataFrame(out, index=index, columns=columns)

    # def update_symbol(self, symbol: tp.Label, **kwargs) -> tp.SeriesFrame:
    #     """Update the symbol.

    #     `**kwargs` will override keyword arguments passed to `GBMData.download_symbol`."""
    #     download_kwargs = self.select_symbol_kwargs(symbol, self.download_kwargs)
    #     download_kwargs['start'] = self.data[symbol].index[-1]
    #     _ = download_kwargs.pop('S0', None)
    #     S0 = self.data[symbol].iloc[-2]
    #     _ = download_kwargs.pop('T', None)
    #     download_kwargs['seed'] = None
    #     kwargs = merge_dicts(download_kwargs, kwargs)
    #     return self.download_symbol(symbol, S0=S0, **kwargs)

    def make(self, _schema: dict = None) -> DataSource:

        """
        Each model is first built using ramdon values for each row/column
        Then for each field that is tagged as "stochastic", that is that each step depends on the previous
        The columns are replaced by arrays of row values which follow a logical progression
        """

        pdf = self.random_pdf()
        print(pdf)
        _fields = fields(self.model)
        for f in _fields:
            name = f.name
            _type = f.type
            if hasattr(f, "metadata"):
                if f.metadata.get("stochastic"):
                    # This field represents a stochastic process
                    # overwrite the noise
                    if _type in [float]:
                        pdf[name] = self.generate_gbm()
                    elif _type in [datetime]:
                        pdf[name] = self.generate_timestamps()
        df = self.pdf_to_sdf(pdf, self.model.get_schema("spark"))
        self.spark_df = df
        return self

        # schema = self.model.get_schema("spark")

        # print(schema)

        # print(data)

        # print(self.model.fake())

        # print("here")
        # print(self.model)

        # # Setup Parameters
        # index = self.context.create_index()
        # print(index)


class REST_DataSource(DataSource):
    """
    Provide methods to access data backed by a REST API
    Creates a Cache locally for repeated calls to the same URL

    * * Fields * *

    context:
        The DataContext to use when pulling data

    endpoint_urls:
        A Pandas DataFrame containing the URLs to be pulled.
        This list of URLs is used for filenames when caching data.
        # TODO The model's "Valid Sources" method is used to return a dataframe from which to call.???

    endpoints:
        A Pandas DataFrame that holds URLs to be called.
        Each model that is accessed using REST contains a function to return such a DataFrame

    """

    def __init__(self, **kwargs) -> None:
        # Take the values from the passed context
        super().__init__(**kwargs)

    def get_rest(
        self, url: str, params: dict = None, sleep=0, root_node=""
    ) -> List[dict]:
        """
        Provides common REST access patterns including
        * Exponential backoff
        * Session Management using Requests

        Establishes and reuses a Session to keep the cookies the same
        """
        if not hasattr(self, "__http_session"):
            self.__http_session = Session()
        _response = self.__http_session.get(url=url, params=params)

        # Everything is Fine
        if _response.status_code == 200:
            print(f"RESPONSE:{_response.status_code} -> got data for {url}")
            results = _response.json()
            if root_node:
                results = results[root_node]
            return results

        # Nothing is Fine
        elif _response.status_code == 404:
            print(f"RESPONSE:{_response.status_code} -> Requested data does not exist")
            return []

        # Try again Later
        else:
            sleep += 1
            if sleep > 5:
                raise Exception(f"Endpoint never returned after {sleep} attempts")
            time.sleep(2**sleep)
            print(f"FAILURE -> retry#{sleep} with URL {url}")
            return self.get_rest(url=url, params=params, sleep=sleep)

    def endpoint_generator(self, endpoint: dict) -> Generator[dict, None, None]:

        """
        Given a configured header
        Retrieve each REST payload and enhance the data as described in the endpoint URLs
        """

        url = endpoint.pop("url")

        if "root_node" in endpoint.keys():
            root_node = endpoint.pop("root_node")
        else:
            root_node = ""
        for record in self.get_rest(url=url, root_node=root_node):
            if endpoint and record:
                record.update(endpoint)
            yield record

        return  # stop generator

    def cache_rest(self) -> List[str]:
        """
        In cases when the REST API has a lot of data use a cache.
        Re-use the URL for file naming.
        If the cache has already been built for a URL then reuse it, otherwise pull it.
        Return a list of files that are accessible locally.
        """

        existing_files = [
            os.path.join(self.local_path, _file)
            for _file in os.listdir(self.local_path)
        ]

        these_files = []

        for index, row in self.endpoint_urls.iterrows():
            endpoint = row.to_dict()
            url = row["url"]
            # Escape non alpha numeric for file system compatibility
            filename = "".join(x if x.isalnum() else "_" for x in url)
            filename = f"{filename}.json"
            full_path = os.path.join(self.local_path, filename)

            if full_path in existing_files:
                # Use the cache
                print(f"Data has already been pulled for {filename}, using cached data")
                these_files.append(full_path)
            else:
                # Pull the data
                with open(full_path, "w") as f:
                    for record in self.endpoint_generator(endpoint=endpoint):
                        # Write newline deliminated JSON, default Spark expectation
                        f.write(json.dumps(record) + " \n")
                these_files.append(full_path)
        return these_files

    def create_sdf(self):

        """
        Wrap the method Spark from Json with the DataModel schema
        Lazy Load a Spark Dataframe
        """

        spark = self.spark_client

        schema = self.model.as_spark_schema()

        streaming = self.streaming

        if self.local_cache:
            # Let Spark Handle the reading of the files
            # by caching locally
            _files = self.cache_rest()
            if not streaming:
                return spark.session.read.schema(schema).json(_files)
            else:
                return spark.session.readStream.schema(schema).json(self.local_path)

        else:
            # Pull fresh data from the URL and don't cache
            # Assumes that everything can fit in memory

            data = self.pdf_from_json(data=self.endpoint_generator())
            return spark.session.createDataFrame(data=data, schema=schema)


class SparkREST_DataSource(REST_DataSource):
    """
    Use a udf to the rest methods to access the endpoints in a distributed way.
    Taking a PDF of endpoints and calling the UDF for each one
    """

    pass

    @classmethod
    def make(cls, context) -> SparkDataFrame:
        """
        Take a data context and populate the Spark DataFrame based on config

        #TODO it is cleaner to dump the entire DataContext, but the assigned SparkClient makes this error
        # Therefore the workaround is to manually specify only the required fields for the Spark UDF
        """

        @dataclass
        class url(DataModel):
            url: str
            context: str

        urls_pdf = cls.endpoint(context=context)
        dc = {
            "start": context.start,
            "end": context.end,
            "endpoint_urls": urls_pdf,
            "model": context.model,
        }

        context_state_str = pickle.dumps(dc)
        context_state_str = base64.b64encode(context_state_str).decode()
        urls_pdf["context"] = context_state_str
        print(urls_pdf)

        # for index, row in urls_pdf.iterrows():
        #     url = row["url"]
        #     executeRestApi(url, context_state_str)

        url_sdf = context.pdf_to_sdf(
            pdf=urls_pdf[["url", "context"]], schema=url.as_spark_schema()
        )

        url_sdf.show()
        url_sdf.printSchema()

        # url_sdf.show()

        # url_sdf = url_sdf.withColumn("context", F.lit(context_state_str))

        ### Get Data ###

        # if not context.local_cache or not data_for_date:
        #     # The data needs to be pulled and is not available locally

        url_sdf = url_sdf.withColumn(
            "response", executeRestApi(F.col("url"), F.col("context"))
        )

        # Take the response DF and explode to a full DF of the passed class

        # schema = StructType([StructField("results", StructType([cls.as_spark_schema()]))
        schema = ArrayType(cls.as_spark_schema())
        df = url_sdf.withColumn(
            "parsed_response",
            F.from_json(
                F.col("response"), schema, {"mode": "PERMISSIVE"}
            ),  # TODO failfast
        )

        df = df.select(F.explode_outer(F.col("parsed_response")).alias("parsed"))
        df = df.select(F.col("parsed.*"))

        return df

    @F.udf(returnType=StringType())
    def executeRestApi(url: str, context: str):
        """
        A UDF which is used in context of a DataFrame "withColumn"

        This allows Spark to delegate this function to workers instead of using only the driver

        Context is passed as a Base64 encoded pickle.

        #TODO can each worker have disk space to write to?
        no it has to fit in the heap space of the worker.

        """

        # Unpack the DataContext passed for this specific URL
        context_dict = pickle.loads(base64.b64decode(context.encode()))

        source = context.source

        pdf = context_dict.pop("endpoint_urls")
        endpoint = pdf[pdf["url"] == url].to_dict("records")[0]

        print(endpoint)

        data = source.get_rest(url)

        return json.dumps([x for x in data])

        # if not data:
        #     return "[]"
        # # data = source.get_rest(url)
        # records = []
        # for x in data:
        #     for y in x:
        #         y.update(pdf.to_dict("records"))
        #         records.append(y)
        # print(records)
        # return json.dumps(records)

        # pdf = source.pdf_from_json(data)

        # spark_df = source.create_sdf()

        # print(context_dict)

        # # data = []
        # api_key = Secrets.get_secrets("polygon").get("key")
        # params = {"apikey": api_key}

        # with requests.get(url, headers=headers, stream=False, params=params) as req:

        #     # print(req.json())
        #     data = req.json()

        #     # for chunk in req.iter_content(chunk_size=4096 * 64):
        #     #     print(chunk)
        #     #     data.append(chunk.json())

        # return json.dumps(data)


class Kafka_DataSource(DataSource):
    """
    A data source that is backed by Kafka
    """

    def __init__(self, **kwargs) -> None:

        values = kwargs["context"].__dict__
        super().__init__(**values)
        self.kafka_client = KafkaClient()
        # Create Spark client if needed, otherwise work in context.
        self.topic = self.model.__name__

        self.kafka_client = KafkaClient()

        # Reformat schema registry location for ABRiS
        self.schema_registry_config = {
            "schema.registry.url": self.kafka_client.schema_registry_dict.get("url")
        }

    def sdf_from_kafka(self):
        """
        Populate Spark DataFrame from Kafka
        """

        return self.spark_client.df_from_kafka(
            topic=self.topic, streaming=self.streaming, start=self.start, end=self.end
        )

    @staticmethod
    def __from_avro(col, config):
        """
        avro deserialize

        :param col (PySpark column / str): column name "key" or "value"
        :param config (za.co.absa.abris.config.FromAvroConfig): abris config, generated from abris_config helper function
        :return: PySpark Column
        """
        jvm_gateway = SparkContext._active_spark_context._gateway.jvm
        abris_avro = jvm_gateway.za.co.absa.abris.avro

        return Column(abris_avro.functions.from_avro(_to_java_column(col), config))

    @staticmethod
    def __from_avro_abris_config(config_map, topic, is_key):
        """
        Create from avro abris config with a schema url

        :param config_map (dict[str, str]): configuration map to pass to deserializer, ex: {'schema.registry.url': 'http://localhost:8081'}
        :param topic (str): kafka topic
        :param is_key (bool): boolean
        :return: za.co.absa.abris.config.FromAvroConfig
        """
        jvm_gateway = SparkContext._active_spark_context._gateway.jvm
        scala_map = jvm_gateway.PythonUtils.toScalaMap(config_map)

        return (
            jvm_gateway.za.co.absa.abris.config.AbrisConfig.fromConfluentAvro()
            .downloadReaderSchemaByLatestVersion()
            .andTopicNameStrategy(topic, is_key)
            .usingSchemaRegistry(scala_map)
        )

    def df_from_kafka(self, topic: str, streaming: False):
        """
        Read a Kafka topic encoded as Avro and return a spark Dataframe object.

        #TODO Support specifying specific offsets to start evaluation from.
        """

        # if model.et_start and model.et_end:
        #     # Use the configured window to limit the offsets parsed on read
        #     pass

        if streaming:
            # Its a Stream!
            # Its a Batch!
            df = (
                self.context.readStream.format("kafka")
                .option(
                    "kafka.bootstrap.servers",
                    self.kafka_client.bootstrap_servers_dict.get("bootstrap.servers"),
                )
                .option("subscribe", topic)
                .option("startingOffsets", "earliest")
                .option("endingOffsets", "latest")
                .load()
            )

        else:
            # Its a Batch!
            df = (
                self.context.read.format("kafka")
                .option(
                    "kafka.bootstrap.servers",
                    self.kafka_client.bootstrap_servers_dict.get("bootstrap.servers"),
                )
                .option("subscribe", topic)
                .option("startingOffsets", "earliest")
                .option("endingOffsets", "latest")
                .load()
            )

        # Get the Avro schema from the schema registry.
        # TODO validate compatibility with the provided schema?
        from_avro_abris_settings = self.__from_avro_abris_config(
            config_map=self.schema_registry_config,
            topic=topic,
            is_key=False,
        )

        return df.withColumn(
            "value", self.__from_avro("value", from_avro_abris_settings)
        )


####################### STREAMING #############################


from pyspark.sql import SparkSession
from pyspark.sql.functions import explode
from pyspark.sql.functions import split


class SparkRateMicroBatch_StreamingSource(DataSource):
    """
    Implements a streaming data source using the Spark 3.3 feature Rate Micro Batch

    This creates an internal stream with reliable and configurable paramters

    """

    def make():
        spark = SparkSession.builder.appName("StructuredNetworkWordCount").getOrCreate()

        query = (
            spark.readStream.format("rate-micro-batch")
            .option("rowsPerBatch", 20)
            .option("numPartitions", 3)
            .load()
            .writeStream.format("console")
            .start()
        )

        query.awaitTermination()


from etl_lib.services.spark.client import SparkClient

from pyspark.sql import SparkSession
import pyspark.sql.functions as F


class Kafka_DataSink(DataContext):

    BOOTSTRAP_SERVERS = "127.0.0.1:3000,127.0.0.1:30001,127.0.0.1:30002"
    TOPIC = "a_test_topic"

    def __init__(self, **kwargs) -> None:

        assert kwargs.get("streaming") == True
        # Assert data.streaming ???
        # Only streaming DataFrames are compatible with this source
        super().__init__(**kwargs)

    def test_spark_to_kafka(df):

        spark = SparkClient("StructuredNetworkWordCount")
        spark = spark.spark_session

        # Format the payload for Kafka
        # All columns
        df = df.withColumn(
            "value",
            F.to_json(F.struct("*")).cast("string"),
        ).select("value")

        # The uniqueness of the message is the hash of the payload itself
        df = df.withColumn("key", F.sha1(F.col("volume")))

        # df.writeStream.format("console").start().awaitTermination()

        df.writeStream.outputMode("append").format("kafka").option(
            "topic", TOPIC
        ).option(
            "kafka.bootstrap.servers", BOOTSTRAP_SERVERS
        ).start().awaitTermination()


if __name__ == "__main__":
    test_spark_to_kafka()
    # vanilla()
