import os
import etl_lib
from pyspark.sql import SparkSession, Column
from pyspark.sql import DataFrame
from pyspark.sql.functions import col, struct, to_json, from_json, decode
from pyspark import SparkContext, SparkConf
from pyspark.sql.column import Column, _to_java_column


import boto3
import os

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

    def __init__(self, app_name: str, config: dict = {}) -> None:

        self.spark_master = "local[*]"
        self._REF = "main"
        self._CATALOG = "blahblahblah"

        #TODO this should be set to the spark master in the K8 cluster
        #self.spark_master = "spark://spark-master-0.spark-headless.default.svc.cluster.local:7077"
        self.app_name = app_name
        self.config = config

        self._BUCKET = os.environ.get("S3_BUCKET_NAME")
        self._SERVER = os.environ.get("NESSIE_SERVER_URI")
        self._S3_ENDPOINT_URL = os.environ.get("S3_ENDPOINT_URL")
        self._S3_ACCESS_KEY_ID = os.environ.get("S3_ACCESS_KEY_ID")
        self._S3_SECRET_ACCESS_KEY = os.environ.get("S3_SECRET_ACCESS_KEY")
        self._PATH = f"s3a://{self._BUCKET}/spark_warehouse"
        self._LOG_PATH = f"s3a://{self._BUCKET}/spark_logs"

        self._NESSIE_VERSION = os.environ.get("NESSIE_VERSION")

        self.create_logging_dir()
        self.session = self.create_spark_session()
        # https://iceberg.apache.org/docs/latest/spark-writes/#type-compatibility



    def create_logging_dir(self):

        s3 = boto3.resource(
            's3',
            endpoint_url=os.environ['S3_ENDPOINT_URL'],
            aws_access_key_id=os.environ['S3_ACCESS_KEY_ID'],
            aws_secret_access_key=os.environ['S3_SECRET_ACCESS_KEY'],
            region_name=os.environ['S3_DEFAULT_REGION']
        )

        bucket = s3.Bucket(os.environ['S3_BUCKET_NAME'])
        dir_obj = None

        try:
            s3.meta.client.head_object(Bucket=os.environ['S3_BUCKET_NAME'], Key='spark_logs/')
            dir_obj = bucket.Object(key='spark_logs/')
        except Exception as e:
            print("Directory does not exist, creating it now.")
            
        if dir_obj is None:
            bucket.put_object(Key='spark_logs/')



    def create_spark_session(self):

        self.create_logging_dir()

        jars_packages = [
            # For Kafka access
            "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
            "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
            "org.apache.kafka:kafka-clients:3.3.1",
            "org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.1",
            "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1",
            "org.apache.hadoop:hadoop-aws:3.3.1",
            "org.apache.hadoop:hadoop-common:3.3.1",
            "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1"
        ]
        jars_packages = ",".join(jars_packages)

        self.config.update({
            "spark.hadoop.fs.s3a.endpoint": self._S3_ENDPOINT_URL,
            "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
            "spark.hadoop.fs.s3a.path.style.access": "true",
            "spark.hadoop.fs.s3a.committer.name": "directory",
            "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
            "spark.hadoop.fs.s3a.access.key": self._S3_ACCESS_KEY_ID,
            "spark.hadoop.fs.s3a.secret.key": self._S3_SECRET_ACCESS_KEY,
            "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
            "spark.sql.execution.arrow.pyspark.enabled": "true",
            "spark.sql.execution.arrow.pyspark.enabled": "true",
            "spark.sql.session.timeZone": "UTC",
            "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint", # TODO when deployed to K8 does this persist?
            F"spark.sql.catalog.{self._CATALOG}.warehouse": self._PATH,
            F"spark.sql.catalog.{self._CATALOG}": "org.apache.iceberg.spark.SparkCatalog",
            F"spark.sql.catalog.{self._CATALOG}.catalog-impl": "org.apache.iceberg.nessie.NessieCatalog",
            F"spark.sql.catalog.{self._CATALOG}.uri": self._SERVER,
            F"spark.sql.catalog.{self._CATALOG}.ref": self._REF,
            F"spark.sql.catalog.{self._CATALOG}.auth_type": "NONE",
            "spark.sql.extensions": "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions",
            "spark.jars.packages": jars_packages,
            # Set logging to use S3
            "spark.eventLog.enabled": "true",
            "spark.eventLog.dir": self._LOG_PATH,
            # "spark.history.fs.logDirectory": F"s3a://{_BUCKET}/",
            "spark.eventLog.rolling.enabled": "true",
            "spark.eventLog.rolling.maxFileSize": "128m",
        })



        spark = SparkSession.builder.master(self.spark_master).appName(self.app_name)

        for key, value in self.config.items():
            spark.config(key, value)

        spark  = spark.getOrCreate()
        spark.sparkContext.setLogLevel("INFO")
        return spark






        # def get_spark_session(app_name: str, conf: dict):
        #     spark = SparkSession.builder.master(self.spark_master).appName(app_name)

        #     for key, value in config.items():
        #         spark.config(key, value)
        #     return spark.getOrCreate()

        # # TODO make calls to this function idempotent
        # self.spark_session = get_spark_session(app_name=app_name, conf=config)
        # self.spark_context = self.spark_session.sparkContext
        # self.spark_context.setLogLevel("INFO")
