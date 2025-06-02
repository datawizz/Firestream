# ETL Lib: An Opinionated Data Platform


ETL Lib is a platform to speed up development of Machine Learning models from ideation to production in batch and streaming by providing a batteries included approach to a modern Big Data stack. Firestream offers a full ML Ops platform that you can run on your laptop, desktop, or public cloud.

Instead of compatibility with the cornucopia of data science projects available today, Firestream aims to combine the best in class open software projects for each primary component of a modern Big Data stack. Where other platforms seek to rebrand open source projects or paywall features, Firestream aims to simply implement the underlying open source software with minimum modifications. Because everything runs in Kubernetes it is easy to deploy Firestream locally or using a cloud provider like GKE or KMS. Because it's all open source it is easy to deploy in air gapped environments or to be modified to suite your use case.

Firestream creates a production ready environment in a single Docker container which includes the core components of a modern Big Data stack:

* Development Environment -> Vs Code + Remote Container
* Container Service -> Moby + Kind + Kubernetes w/ DinD
* Pub/Sub -> Kafka
* Stream Processing -> Spark
* Batch Processing -> Spark
* Metadata Persistence -> PostgreSQL
* Object Storage -> MinIO (S3) via Hadoop
* Storage Format -> Parquet
* ML Model Tracking / Deployment -> ML Flow
* Python API -> Firestream

## **Installation**

Prerequisites:
* Debian 11+ / Ubuntu 18.02+ with Docker installed

```
git clone https://github.com/symtrade/firestream && \
cd firestream && \
sh bootstrap.sh
```

This will download the repo and begin building the resources. As services are started you will see output in the terminal listing the ports to access each service as http://SERVICE:PORT which will be automatically forwarded to the terminal that invoked bootstrap.sh.

Firestream makes extensive use of IpTables for internal networking and modification of /etc/hosts for internal DNS and has only been tested on debian 11+.


## Architecture

![Screenshot](images/Firestream.png)

Firestream uses Kafka for pub/sub message passing between Spark executors. Microservices are run as pods for bespoke data connections while Spark Applications are used for processing both streaming and batch data. Using the Firestream API, models are declared in the abstract. A model called with streaming=True will be executed against the Kafka cluster. If streaming=False the model will be excuted as a spark batch operation against external data or data stored internally in MinIO S3 object store. If --cache is set the model will be materialized as a parquet file increasing re-processing batch speed or checkpointed if streaming. 

## Monitoring

Firestream uses https://github.com/txn2/kubefwd to make all resources available via port forwarding to the computer that is ssh'ed into the Firestream main container.


The state of streaming and batch jobs can be viewed using the Spark History Server at : http://spark_history:5001

Kafka can be monitored using Kafka Manager at : 

MinIO contains a web server for it's status available at : http://minio:9000

bootstrap.sh launches some heartbeat jobs for a smoketest.
It also runs each of the tests in ./tests




## Example Model

Firestream provides a Python API for **Model Driven Engineering** where the transformations are highlighted and the developer (mostly) does not *need* to reason about the state of the underlying processing. This *reason when you want to* approach is considered a feature of Firestream.


```
"""
Define a pipeline as a series of models that are both distinct steps and are interconnected.

"""


from etl_lib import DataModel, DataFactory
from etl_lib.sources import Random_DataSource
from etl_lib.sinks import Kafka_DataSink
from etl_lib.types import (
    Datetime_Type,
    Timedelta_Type,
    BrownianMotion_Type,
    Float_Type,
    Integer_Type,
    String_Type,
    Array_Type
    )

@DataModel
class StockTrades:
    symbol: String_Type
    event_time: Datetime_Type
    price: Float_Type
    volume: Integer_Type

    def make(context):
        df = context.make(context.model)
        return df


@DataModel
class StockCandle:
    symbol: String_Type
    event_time: Datetime_Type
    high: Float_Type
    low: Float_Type
    open: Float_Type
    close: Float_Type

    def make(context):
        df = StockTrades.make(context)

        # Pandas API
        df.resample('1H').agg({
            'open':'first',
            'high':'max',
            'low':'min',
            'close':'last'
        })

        return df

with DataFactory(
    start="2021-01-01",
    end="2021-12-31",
    intervals=[timedelta(days=1)],
    symbols=[SPY, TSLA, XOM]
    ) as factory:

    factory.make(
        model = StockCandle,
        source = factory.make(
            model=StockTrades,
            source=BrownianMotion_DataSource,
            sink=Kafka_DataSink
        )
    )

    factory.run()

```




## Motivation

The modern Data Stack requires a diverse set of technologies and skillsets that generally require a team to develop and operate successfully. Platforms exist which combine open source projects with closed source resources to create vendor locked environments. Sometimes you need to inspect every aspect of the Machine Learning process and nothing like local or single server development really gives you the same access to the components. When it is time to deploy there should be no friction and the compute / storage resources should be cloud scale. 

The process of "Idea" to "Production" usually looks like this:

1. Have an idea 💡
2. Create a model prototype against a subset of data
3. Test/Train the model prototype against all of your data
4. Rewrite the model to work in production, expose a REST endpoint
5. Rewrite the model to work on streaming data, integrate with Pub/Sub
6. Deploy the model in production to process live data
7. Monitor the model for errors and SLA guarantees
8. Maintain and Improve the model, sometimes with breaking changes
9. Have an idea for a prototype that uses this model as input, see step 1.

The existing toolbox of open source "Data Pipelines" is a cornucopia of options to accomplish this process by abstracting away key parts. Enterprise options generally reduce complexity by providing simplified (proprietary) APIs for portions of this process while selling compute resources. Open source projects generally choose a part of the process to focus on and ignore the rest. Projects which attempt to tackle the full process make key parts inaccessible which prevents flexibility in deployment that might be required for your specific use case.

Firestream approaches this problem by leveraging open source projects to create a ensamble Data Infrustructure Stack, wrapped in a pythonic API to enable *Model Driven Engineering* where models are written once and work everywhere.



## Deployment

```
# Run once as a batch job
python example_model.py batch

# Run as a batch job on a daily schedule
python example_model.py batch daily

# Run persistently as a stream job
python example_model.py stream

```


## Under the Hood

ETL Lib uses Kind to create a local Kubernetes cluster running inside Docker. Using Helm the Bitnami Charts for Kafka are deployed with minor modifications. Using Helm the Spark_on_K8 operator is used for persistant jobs. A Hadoop HDFS instance is created in the K8 cluster as well for persistent storage. Finally a Jupyter Notebook server is launched in K8 for exploratory analysis.

ETL Lib uses git submodules and shallow clone to pull specific required directories.

## Goals

Building dynamic composistions of data in the abstract with seemless deployment to production and super fast execution via Spark. ETL Lib defines a set of objects which provide a common API for accessing and blending data in motion, at rest, internal, and external, in development and straight to production.

Firestream is a libary to encompass the full modern data stack. It takes from the best open source projects and combines their strengths with the aim to dazzle.

- Small - only ~ xxx lines of code
- Fast - Uses PySpark under the hood
- Flexible - Python idioms with strong types. Enables quick prototyping and deployment to production with the same code and schema evolution.
- Streaming - Uses Spark Streaming and payload hashing with Kafka to enable exactly once processing inside expected SLA
- Optimized - Includes partition mapping in data definition which is seemlessly deployed in a Kafka cluster and locally using Parquet for fast iterative development without reading excess data.
- Big - Use sharded partition for abstract definitions of data that enable fast access to S3 API by reducing time-to-list

## Comparisons to other projects


* dbt - Does not support streaming
* ML Flow / Kubeflow - Platform with no API
* Hopsworks - Paywalled for advanced usage
* Databricks - Requires EULA that gives them your data
* 

### ETL Lib Objects

DataModel:
    The DataModel is the core object of ETL Lib.
    The DataModel is a simple Python Dataclass with a "make" method. The Make method defines the steps to create a instace of that DataModel as a Spark DataFrame.
    Fields within the DataModel may be other DataModels (composition) or built in Python types. All fields are thus strongly typed.
    Fields are based on the <https://avro.apache.org/docs/> standard. ETL Lib fields support all data structures that the Avro standard supports.

    Every DataModel is responsible for two things:

        1. Provide a "Make" method which returns a DataFrame of that model, given a DataContext. The DataContext will detail the mode (streaming/batch) the start and end timestamps, etc to fully parameterize the DataModel for all environments. This allows for straight forward testing.

        2. Provide an API to the make method (as python arguments) which creates the DataModel under different conditions. i.e. a Make method could be provided any DataFrame to substitute for a source so long as the provided DataFrame contains the expected columns.

    The Transform method

## Core Objects

- DataSource:
    An object that can be passed as an argument to a DataModel's "Make" method in lieu of a Spark DataFrame. This enables data to be sourced from the external world (REST) or generated (Geometric Brownian Motion), or define your own source based on your environment. DataModels which are made using a DataSource return an instance of themselves just like any other method of making a DataModel.

- DataSink:
    Provides a standard interface to write out data.
    Supported data destinations include:
    1. To the console (useful for development)
    2. Locally (useful for single box testing, ML Training, and development)
    3. To the Kafka Cluster in Kubernetes (production, testing)

- DataContext:
    An object which contains all parameters needed to run a DataModel in development, testing, ML training, or production settings. Includes start and end timestamps, streaming vs batch, the DataSink, etc.
    Includes methods to retrieve secrets required to access private data sources.

- DataFactory:
    Takes as input a abstract syntax tree of DataModels and arguments to their make method.
    Enables complex composistion of models via a single mapping object.

- DataType:
    Defines a custom type for a column of data. Types can include instructions on how to construct them (Brownian Motion, Normal Distribution, compressed image, etc)





# Known Issues

1. Spark 3.3.1 + Hadoop 3.3.4 issue a warning about logging in S3. "WARN s3a.S3ABlockOutputStream: Application invoked the Syncable API against stream writing"
This can safely be ignored as logs are indeed written even though the "file system" doesn't support flush symantics. https://issues.apache.org/jira/browse/HADOOP-17847




2. Kafka as packaged by Bitnami reports odd addresses for the service when installed using helm. The correct one saved in the project's config is kafka.default.svc.cluster.local which is accessible from within the dev container pod based on DNS hackz



# DNS

The DNS of the DevContainer is set in the DockerCompose file to a value that is set in the Kind cluster config, 10.96.0.10. This allows the DevContainer to "reach" out to the Docker host and route DNS requests to the Kind Control Plane, which makes service hostnames resolvable by the dev container. However, if the Kind Control Plane is not booted there is no valid DNS in the container. This makes VS Code plugins fail to install, and could be confusing.

Ideally, the K8 DNS can be used as a secondary DNS endpoint (without a performacne hit?) which would allow regular addresses to be resolved without the Kind cluster being up.