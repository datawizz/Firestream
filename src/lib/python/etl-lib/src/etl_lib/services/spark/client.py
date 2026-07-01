import os
from typing import Any, Dict, Optional

import boto3
from pyspark.sql import DataFrame, SparkSession


class SparkClient:
    """Configure and hold a SparkSession with Delta + S3 + LakeFS support.

    Connection details for S3 and LakeFS are read from environment variables.
    Construction creates the session immediately so the caller can use
    ``self.spark_session`` directly.

    Required environment variables when the corresponding backend is used:
      * S3:     ``S3_LOCAL_*`` (or ``S3_CLOUD_*`` when ``storage_location='cloud'``)
      * LakeFS: ``LAKEFS_ENDPOINT_URL``, ``LAKEFS_ACCESS_KEY``, ``LAKEFS_SECRET_KEY``
    """

    def __init__(
        self,
        app_name: str = "etl-lib",
        config: Optional[Dict[str, Any]] = None,
        master: str = "local[*]",
        catalog: str = "delta",
        branch: Optional[str] = None,
        storage_location: str = "local",
    ) -> None:
        self.app_name = app_name
        self.config: Dict[str, Any] = dict(config or {})
        self.spark_master = master
        self.CATALOG = catalog
        self.BRANCH = branch or os.environ.get("DEPLOYMENT_MODE", "main")
        self.storage_location = storage_location

        self._load_s3_credentials()
        self._load_lakefs_credentials()

        self.S3_PATH = f"s3a://{self.S3_BUCKET_NAME}/warehouse"
        self.LOG_PATH = f"s3a://{self.S3_BUCKET_NAME}/spark_logs/"

        self.create_logging_dir()

        self.spark_session = self.create_spark_session()
        self.spark_context = self.spark_session.sparkContext

    def _load_s3_credentials(self) -> None:
        prefix = "S3_LOCAL_" if self.storage_location == "local" else "S3_CLOUD_"
        self.S3_ENDPOINT_URL = os.environ.get(f"{prefix}ENDPOINT_URL")
        self.S3_ACCESS_KEY_ID = os.environ.get(f"{prefix}ACCESS_KEY_ID")
        self.S3_SECRET_ACCESS_KEY = os.environ.get(f"{prefix}SECRET_ACCESS_KEY")
        self.S3_BUCKET_NAME = os.environ.get(f"{prefix}BUCKET_NAME")
        self.S3_DEFAULT_REGION = os.environ.get(f"{prefix}DEFAULT_REGION")

    def _load_lakefs_credentials(self) -> None:
        self.LAKEFS_ENDPOINT_URL = os.environ.get("LAKEFS_ENDPOINT_URL")
        self.LAKEFS_ACCESS_KEY = os.environ.get("LAKEFS_ACCESS_KEY")
        self.LAKEFS_SECRET_KEY = os.environ.get("LAKEFS_SECRET_KEY")

    def set_bucket(self, storage_location: str) -> None:
        """Switch between the local and cloud S3 credential sets at runtime."""
        self.storage_location = storage_location
        self._load_s3_credentials()
        self.S3_PATH = f"s3a://{self.S3_BUCKET_NAME}/warehouse"
        self.LOG_PATH = f"s3a://{self.S3_BUCKET_NAME}/spark_logs/"

    def create_logging_dir(self) -> None:
        """Ensure the Spark event-log bucket and prefix exist on S3."""
        if not self.S3_BUCKET_NAME:
            return
        s3 = boto3.resource(
            "s3",
            endpoint_url=self.S3_ENDPOINT_URL,
            aws_access_key_id=self.S3_ACCESS_KEY_ID,
            aws_secret_access_key=self.S3_SECRET_ACCESS_KEY,
            region_name=self.S3_DEFAULT_REGION,
        )
        if s3.Bucket(self.S3_BUCKET_NAME) not in s3.buckets.all():
            s3.create_bucket(Bucket=self.S3_BUCKET_NAME)
        try:
            s3.meta.client.head_object(Bucket=self.S3_BUCKET_NAME, Key="spark_logs/")
        except Exception:
            s3.Bucket(self.S3_BUCKET_NAME).put_object(Key="spark_logs/")

    def config_spark(self) -> None:
        """Merge the library's default Spark configuration into ``self.config``."""
        jars_packages = ",".join(
            [
                "org.apache.spark:spark-sql-kafka-0-10_2.12:3.4.1",
                "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.4.1",
                "org.apache.kafka:kafka-clients:3.4.1",
                "org.apache.hadoop:hadoop-aws:3.3.1",
                "org.apache.hadoop:hadoop-common:3.3.1",
                "org.apache.spark:spark-hadoop-cloud_2.12:3.4.1",
                "io.delta:delta-core_2.12:2.4.0",
                "io.lakefs:hadoop-lakefs-assembly:0.1.15",
            ]
        )
        self.jars_packages = jars_packages

        self.config.update(
            {
                "spark.jars.packages": jars_packages,
                "spark.hadoop.fs.lakefs.impl": "io.lakefs.LakeFSFileSystem",
                "spark.hadoop.fs.lakefs.endpoint": self.LAKEFS_ENDPOINT_URL,
                "spark.hadoop.fs.lakefs.connection.ssl.enabled": "false",
                "spark.hadoop.fs.lakefs.access.key": self.LAKEFS_ACCESS_KEY,
                "spark.hadoop.fs.lakefs.secret.key": self.LAKEFS_SECRET_KEY,
                "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
                "spark.hadoop.fs.s3a.access.key": self.S3_ACCESS_KEY_ID,
                "spark.hadoop.fs.s3a.secret.key": self.S3_SECRET_ACCESS_KEY,
                "spark.hadoop.fs.s3a.endpoint": self.S3_ENDPOINT_URL,
                "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
                "spark.hadoop.fs.s3a.path.style.access": "true",
                "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
                "spark.sql.catalog.spark_catalog": "org.apache.spark.sql.delta.catalog.DeltaCatalog",
                "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
                "spark.sql.execution.arrow.pyspark.enabled": "true",
                "spark.sql.session.timeZone": "UTC",
                f"spark.sql.catalog.spark_catalog.warehouse": self.S3_PATH,
                "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint",
                "spark.eventLog.rolling.enabled": "true",
                "spark.eventLog.rolling.maxFileSize": "128m",
                "spark.driver.memory": "4g",
                "spark.executor.memory": "4g",
                "spark.offHeap.enabled": "true",
                "spark.memory.offHeap.size": "4g",
            }
        )

    def create_spark_session(self) -> SparkSession:
        builder = SparkSession.builder.master(self.spark_master).appName(self.app_name)
        self.config_spark()
        for key, value in self.config.items():
            builder = builder.config(key, value)
        spark = builder.getOrCreate()
        spark.sparkContext.setLogLevel(self.config.get("spark.log.level", "INFO"))
        return spark

    def read(self, format: str, **kwargs):
        return self.spark_session.read.format(format).options(**kwargs)

    def write(self, df: DataFrame, format: str, **kwargs):
        return df.write.format(format).options(**kwargs)

    def read_stream(self, format: str, **kwargs):
        return self.spark_session.readStream.format(format).options(**kwargs)
