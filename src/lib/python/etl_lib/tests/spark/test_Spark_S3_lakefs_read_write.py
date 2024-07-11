# #!/usr/bin/env python3

# """
# This test script tests using Spark to read and write to S3.
# It makes use of the Hadoop S3A connector and the Spark S3A connector.
# Data is written in Parquet format.
# """

# from pyspark import SparkConf
# from pyspark.sql import SparkSession
# import os


# bucket = os.environ["S3_LOCAL_BUCKET_NAME"]
# _PATH = f"s3a://{bucket}/example/data/cities_parquet"
# _DATA = "/workspace/_WIP/test_WIP/example_data/cities.csv"
# _ACCESS_KEY = os.environ["S3_LOCAL_ACCESS_KEY_ID"]
# _SECRET_KEY = os.environ["S3_LOCAL_SECRET_ACCESS_KEY"]
# _ENDPOINT_URL = os.environ["S3_LOCAL_ENDPOINT_URL"]

# config = {
#     "spark.sql.execution.arrow.pyspark.enabled": "true",
#     "spark.hadoop.fs.s3a.endpoint": _ENDPOINT_URL,
#     "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
#     "spark.hadoop.fs.s3a.path.style.access": "true",
#     "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
#     "spark.hadoop.fs.s3a.access.key": _ACCESS_KEY,
#     "spark.hadoop.fs.s3a.secret.key": _SECRET_KEY,
#     "spark.jars.packages": "org.apache.hadoop:hadoop-aws:3.3.4,org.apache.hadoop:hadoop-common:3.3.4,org.apache.spark:spark-hadoop-cloud_2.12:3.4.1",
#     "spark.sql.parquet.mergeSchema": "false",
#     "spark.sql.parquet.filterPushdown": "true",
#     "spark.sql.hive.metastorePartitionPruning": "true",
#     "spark.hadoop.fs.s3a.committer.name": "directory",
#     "spark.sql.sources.commitProtocolClass": "org.apache.spark.internal.io.cloud.PathOutputCommitProtocol",
#     "spark.sql.parquet.output.committer.class": "org.apache.spark.internal.io.cloud.BindingParquetOutputCommitter",
# }


# def create_spark_session():

#     app_name = "test"

#     spark = SparkSession.builder.master("local[1]").appName(app_name)

#     for key, value in config.items():
#         spark.config(key, value)
#     return spark.getOrCreate()


# import unittest

# class SparkContextBaseTestCase(unittest.TestCase):
#     @classmethod
#     def setUpClass(cls):
#         cls.spark = create_spark_session()
    

#     @classmethod
#     def tearDownClass(cls):
#         cls.spark.stop()



# class SparkAppTest(SparkContextBaseTestCase):


#     def test_write(self):

#         sc = self.spark.sparkContext
#         self.spark.sparkContext.setLogLevel("WARN")
#         # print(f"Hadoop version = {sc._jvm.org.apache.hadoop.util.VersionInfo.getVersion()}")

#         df = self.spark.read.csv(_DATA, header=True, inferSchema=True)

#         df.show()

#         df.write.format("parquet").mode("overwrite").save(_PATH)






#     def test_read(self):

#         df = self.spark.read.parquet(_PATH)

#         df.show()
#         df.printSchema()

#         df = df.select("State").where(df.State == "CA")

#         df.show()



# if __name__ == '__main__':
#     import pytest
#     pytest.main([__file__])


import os
os.environ["LAKEFS_ENDPOINT_URL"] = "http://lakefs.default.svc.cluster.local:80/api/v1"
os.environ["LAKEFS_REPO"] = "firestream"
os.environ['LAKEFS_BRANCH'] = "main"
os.environ['LAKEFS_BUCKET'] = "firestream"
os.environ["LAKEFS_ACCESS_KEY"] = "AKIAJV77CIW6QKTEQWSQ"
os.environ["LAKEFS_SECRET_KEY"] = "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"


from etl_lib.services.spark.client import SparkClient
import os

def create_spark_client():
    return SparkClient(app_name="Pytest", config={}, storage_location="local")

def main():
    spark_client = create_spark_client()
    spark_client.spark_context.setLogLevel("WARN")

    bucket = os.environ["S3_LOCAL_BUCKET_NAME"]
    _PATH = f"s3a://{bucket}/example/data/cities_parquet"
    _DATA = "/workspace/src/lib/python/etl_lib/tests/example_data/cities.csv"
    
    # Test write
    df = spark_client.spark_session.read.csv(_DATA, header=True, inferSchema=True)
    df.show()
    df.write.format("parquet").mode("overwrite").save(_PATH)
    
    # Test read
    df = spark_client.spark_session.read.parquet(_PATH)
    df.show()
    df.printSchema()

    df = df.select("State").where(df.State == "CA")
    df.show()

    spark_client.stop()

if __name__ == '__main__':
    main()
