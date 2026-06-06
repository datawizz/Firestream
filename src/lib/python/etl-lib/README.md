# etl-lib

Typed Python interface for working with the data services in the Firestream
repository: Spark, Kafka, S3/Delta via the Hadoop S3A connector, LakeFS, and
GCP Secret Manager. Used from Airflow DAGs and ad-hoc ETL jobs.

> **Status:** in active development. The high-level pipeline API
> (`DataFactory.run`, `DataModel.make`, the streaming sources, S3/Kafka sinks)
> is published as `NotImplementedError` so the contract is visible to callers
> and typecheckers — but the wiring is not finished. The lower-level helpers
> (`SparkClient`, `KafkaClient`, REST source, Brownian-motion generator, type
> conversion utilities) are usable today.

## Install

```bash
uv pip install -e .
# or
pip install -e .
```

Python 3.12.10 is required.

## What ships today

### Working

| Module                                   | What it does                                                         |
| ---------------------------------------- | -------------------------------------------------------------------- |
| `etl_lib.DataContext`                    | Holds model + time-window + Spark config; pandas↔Spark conversion.   |
| `etl_lib.DataModel`                      | Pydantic base with `json_schema()` / `avro_schema()` export.         |
| `etl_lib.services.SparkClient`          | Configured `SparkSession` with Delta + S3A + LakeFS.                 |
| `etl_lib.services.KafkaClient`          | Avro `SerializingProducer`, schema-registry helpers, admin client.   |
| `etl_lib.source.REST_DataSource`         | HTTP GET with exponential backoff and local file cache.              |
| `etl_lib.source.BrownianMotion_DataSource.generate_gbm` | Geometric Brownian Motion price series. |
| `etl_lib.source.BrownianMotion_DataSource.generate_timestamps` | Jittered event timestamps. |
| `etl_lib.sink.Console_DataSink`          | `.show()` + `.printSchema()` for development.                        |
| `etl_lib._utils.types.types`             | `@struct` decorator + Python-type → Spark `StructType` inference.    |
| `etl_lib._utils.secrets.Secrets`         | GCP Secret Manager wrapper (ADC or service-account JSON).            |
| `etl_lib._utils.git_revision`            | Current `HEAD` SHA (full and short).                                 |
| `etl_lib._utils.broadcast_join`          | Bucketed event-time stream-stream join helper.                       |

### Declared but `NotImplementedError`

These appear in the public API so downstream code can typecheck against the
intended shape. Calling them raises `NotImplementedError`:

- `DataModel.make` (abstract; subclasses must implement)
- `DataFactory.run`
- `DataContext.__len__`
- `DataIndex.make`, `DataHeader.make`, `DataFooter.make`
- `Random_DataSource.random_data` (depends on `DataModel.make_one`)
- `BrownianMotion_DataSource.make`
- `SparkDataFrame_DataSource.make`
- `SparkREST_DataSource.make`
- `Kafka_DataSource.make`, `Kafka_DataSource.df_from_kafka`
- `SparkRateMicroBatch_StreamingSource.make`
- `S3_DataSink.sink`
- `Console_DataSink.sink` for streaming DataFrames

### Not shipping

Previously stubbed and now removed entirely:

- `etl_lib.services.solr`
- `etl_lib.services.JDBC`
- `etl_lib.services.pytorch` (Petastorm + PyTorch integration)
- `Local_DataSink`, `GoogleCloudStorage`, `Kafka_DataSink`, `TestSink`

## Quickstart

```python
from etl_lib import DataContext
from etl_lib.services.spark.client import SparkClient

spark = SparkClient(app_name="hello")

ctx = DataContext(
    model=None,
    start="2024-01-01",
    end="2024-01-02",
    spark_client=spark,
)

# Pandas → Spark
import pandas as pd
pdf = pd.DataFrame({"x": [1, 2, 3]})
sdf = spark.spark_session.createDataFrame(pdf)
sdf.show()
```

Define a typed schema:

```python
from etl_lib import DataModel
from typing import Optional

class Trade(DataModel):
    symbol: str
    price: float
    volume: int
    nickname: Optional[str] = None

print(Trade(symbol="AAPL", price=190.0, volume=100).get_schema("AVRO"))
```

Generate a Geometric Brownian Motion price series:

```python
from datetime import timedelta
import pandas as pd
from etl_lib.source import BrownianMotion_DataSource

src = BrownianMotion_DataSource.__new__(BrownianMotion_DataSource)
src.start = pd.Timestamp("2024-01-01T00:00:00", tz="UTC")
src.end = src.start + pd.Timedelta(seconds=60)
src.intervals = [timedelta(seconds=1)]
src.length = 60
src.seed = 42

series = src.generate_gbm(mu=0.1, sigma=0.1, p0=100.0)
```

Once `DataModel.make_one` is implemented, the same source will be reachable
through the higher-level `.make()` pipeline.

## Environment variables

`SparkClient` reads connection details from the environment so credentials
never need to land in source. Set the ones relevant to your deployment:

| Variable                          | Used by                                                  |
| --------------------------------- | -------------------------------------------------------- |
| `S3_LOCAL_ENDPOINT_URL`           | S3A endpoint (when `storage_location="local"`)           |
| `S3_LOCAL_ACCESS_KEY_ID`          | S3A access key                                           |
| `S3_LOCAL_SECRET_ACCESS_KEY`      | S3A secret                                               |
| `S3_LOCAL_BUCKET_NAME`            | Warehouse bucket                                         |
| `S3_LOCAL_DEFAULT_REGION`         | Bucket region                                            |
| `S3_CLOUD_*`                      | Same set, used when `storage_location="cloud"`           |
| `LAKEFS_ENDPOINT_URL`             | LakeFS API endpoint                                      |
| `LAKEFS_ACCESS_KEY`               | LakeFS access key                                        |
| `LAKEFS_SECRET_KEY`               | LakeFS secret                                            |
| `KAFKA_BOOTSTRAP_SERVERS`         | Kafka bootstrap (default points at the local K3D cluster)|
| `KAFKA_SCHEMA_REGISTRY_URL`       | Schema registry URL                                      |
| `DEPLOYMENT_MODE`                 | Used as the default catalog branch (`main` if unset)     |
| `GCP_PROJECT_ID`                  | Used by `Secrets` for Secret Manager calls               |
| `GOOGLE_APPLICATION_CREDENTIALS_PATH` / `GCP_CRED_RAW` | Fallback service-account credentials |

## Tests

```bash
pytest                  # unit tests only (default — integration is excluded)
pytest -m integration   # requires a live K3D cluster + S3/Kafka
```

Unit tests cover the type-conversion utilities, the git revision helpers, and
the Brownian-motion generator. They run without any external services.
