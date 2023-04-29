import os
import etl_lib
from pyspark.sql import SparkSession, Column
from pyspark.sql import DataFrame
from pyspark.sql.functions import col, struct, to_json, from_json, decode
from pyspark import SparkContext, SparkConf
from pyspark.sql.column import Column, _to_java_column


from pyspark.sql.functions import sha2, concat_ws, bin, hash

import time
from datetime import datetime, timedelta
import json


from etl_lib.services.kafka.client import KafkaClient

# from tinsel import struct, transform
import numpy as np
import pandas as pd


# TODO The key setting is wonky AF
# TODO The schema in schema registry does not appear to get set
# TODO The topic retention time, key col, timestamp, etc should be set when producing records.
# TODO the jars required to run this are sourced from the internet. This should be built from source as part of the project
# TODO convert this to use a dataclass for all schema references
# TODO The schema registry does not evolve when adding a new field

# TODO https://spark.apache.org/docs/3.1.1/api/python/reference/api/pyspark.sql.streaming.DataStreamWriter.foreachBatch.html
# TODO use the Kubernetes executor for all spark jobs
# TODO build the jars into the dockerfiles served to Kubernetes to minimize spin up and network traffic on executors
from pyspark.sql import SQLContext


from pyspark import SparkConf
from pyspark.sql import SparkSession
import os


class SparkClient:

    """
    Configures a Spark SQL Context for Batch and Streaming DataSource(s) and DataSink(s)

    Logs actions to a local S3 instance

    """

    def __init__(self, app_name: str) -> None:

        self.spark_master = "local[*]"

        # TODO download these Jars in the Dockerfile to eliminate repeated downloads in Driver and Executor pods
        jars_packages = [
            # For S3 access
            "org.apache.hadoop:hadoop-aws:3.3.1",
            "org.apache.hadoop:hadoop-common:3.3.1",
            "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1",
            # For Kafka access
            "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
            "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
            "org.apache.kafka:kafka-clients:3.3.1",
            # For Avro in Kafka
            "org.apache.spark:spark-avro_2.12:3.2.1",
            "za.co.absa:abris_2.12:6.2.0",
        ]

        repos = [
            "https://packages.confluent.io/maven",
            "https://mvnrepository.com/artifact/za.co.absa/abris",
        ]
        jars = ",".join(jars_packages)
        repos = ",".join(repos)

        args = f"--packages {jars} --repositories {repos} pyspark-shell"
        os.environ["PYSPARK_SUBMIT_ARGS"] = args

        config = {
            # "spark.kubernetes.namespace": "spark",
            # "spark.kubernetes.container.image": "itayb/spark:3.1.1-hadoop-3.2.0-aws",
            # "spark.executor.instances": "2",
            # "spark.executor.memory": "1g",
            # "spark.executor.cores": "1",
            # "spark.driver.blockManager.port": "7777",
            # "spark.driver.port": "2222",
            # "spark.driver.host": "jupyter.spark.svc.cluster.local",
            # "spark.driver.bindAddress": "0.0.0.0",
            "spark.hadoop.fs.s3a.endpoint": os.environ.get("AWS_ENDPOINT_URL"),
            "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
            "spark.hadoop.fs.s3a.path.style.access": "true",
            # "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
            "spark.hadoop.com.amazonaws.services.s3.enableV4": "true",
            # "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.AnonymousAWSCredentialsProvider",
            "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
            "spark.hadoop.fs.s3a.access.key": os.environ.get("AWS_ACCESS_KEY_ID"),
            "spark.hadoop.fs.s3a.secret.key": os.environ.get("AWS_SECRET_ACCESS_KEY"),
            "spark.hadoop.parquet.enable.summary-metadata": "false",
            "spark.sql.parquet.mergeSchema": "false",
            "spark.sql.parquet.filterPushdown": "true",
            "spark.sql.hive.metastorePartitionPruning": "true",
            "spark.hadoop.fs.s3a.committer.name": "directory",
            "spark.sql.sources.commitProtocolClass": "org.apache.spark.internal.io.cloud.PathOutputCommitProtocol",
            "spark.sql.parquet.output.committer.class": "org.apache.spark.internal.io.cloud.BindingParquetOutputCommitter",
            "spark.sql.execution.arrow.pyspark.enabled": "true",
            "spark.sql.session.timeZone": "UTC",
            "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint",  # Set this as a local directory instead of S3 for speed?
            # Set logging to use S3
            "spark.eventLog.enabled": "true",
            "spark.eventLog.dir": "s3a://logs/spark/",
            "spark.history.fs.logDirectory": "s3a://logs/spark/",
            "spark.eventLog.rolling.enabled": "true",
            "spark.eventLog.rolling.maxFileSize": "128m",
        }

        def get_spark_session(app_name: str, conf: dict):
            spark = SparkSession.builder.master(self.spark_master).appName(app_name)

            for key, value in config.items():
                spark.config(key, value)
            return spark.getOrCreate()

        # TODO make calls to this function idempotent
        self.spark_session = get_spark_session(app_name=app_name, conf=config)
        self.spark_context = self.spark_session.sparkContext
        self.spark_context.setLogLevel("INFO")
