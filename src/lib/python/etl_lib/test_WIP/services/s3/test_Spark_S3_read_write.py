from pyspark import SparkConf
from pyspark.sql import SparkSession
import os


# For Minio

# os.environ['AWS_ACCESS_KEY_ID'] = 'dev_admin'
# os.environ['AWS_SECRET_ACCESS_KEY'] = 'dev_admin'
# os.environ['AWS_DEFAULT_REGION'] = 'us-west-2'
# os.environ['AWS_ENDPOINT_URL'] = 'minio.default.svc.cluster.local:9000'
bucket = os.environ.get("S3_BUCKET_NAME")
_PATH = f"s3a://{bucket}/example/data/cities_parquet"
_DATA = "/workspace/src/lib/python/etl_lib/tests/example_data/cities.csv"

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
    "spark.sql.execution.arrow.pyspark.enabled": "true",
    "spark.hadoop.fs.s3a.endpoint": os.environ.get("S3_ENDPOINT_URL"),
    "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
    "spark.hadoop.fs.s3a.path.style.access": "true",
    # "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
    # "spark.hadoop.com.amazonaws.services.s3.enableV4": "true",
    # "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.AnonymousAWSCredentialsProvider",
    "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
    "spark.hadoop.fs.s3a.access.key": os.environ.get("S3_ACCESS_KEY_ID"),
    "spark.hadoop.fs.s3a.secret.key": os.environ.get("S3_SECRET_ACCESS_KEY"),
    "spark.jars.packages": "org.apache.hadoop:hadoop-aws:3.3.1,org.apache.hadoop:hadoop-common:3.3.1,org.apache.spark:spark-hadoop-cloud_2.12:3.3.1",
    # "spark.hadoop.parquet.enable.summary-metadata": "false",
    "spark.sql.parquet.mergeSchema": "false",
    "spark.sql.parquet.filterPushdown": "true",
    "spark.sql.hive.metastorePartitionPruning": "true",
    "spark.hadoop.fs.s3a.committer.name": "directory",
    "spark.sql.sources.commitProtocolClass": "org.apache.spark.internal.io.cloud.PathOutputCommitProtocol",
    "spark.sql.parquet.output.committer.class": "org.apache.spark.internal.io.cloud.BindingParquetOutputCommitter",
}


def create_spark_session():

    app_name = "test"

    spark = SparkSession.builder.master("local[*]").appName(app_name)

    for key, value in config.items():
        spark.config(key, value)
    return spark.getOrCreate()


# import pyspark
# import os


# spark = (
#     pyspark.sql.SparkSession.builder.master("local[*]")
#     .config("spark.driver.memory", "1G")
#     .config("spark.executor.memory", "1G")
#     .config("spark.hadoop.fs.s3a.access.key", os.environ.get("AWS_ACCESS_KEY_ID"))
#     .config("spark.hadoop.fs.s3a.secret.key", os.environ.get("AWS_SECRET_ACCESS_KEY"))
#     .getOrCreate()
# )


def test_write(spark):

    sc = spark.sparkContext
    spark.sparkContext.setLogLevel("WARN")
    # print(f"Hadoop version = {sc._jvm.org.apache.hadoop.util.VersionInfo.getVersion()}")

    # Read the known good data
    #TODO make this file read relative to the files instead of the fully qualified domain name
    df = spark.read.csv(_DATA, header=True, inferSchema=True)

    df.show()

    df.write.format("parquet").mode("overwrite").save(_PATH)






def test_read(spark):

    df = spark.read.parquet(_PATH)

    df.show()
    df.printSchema()

    df = df.select("State").where(df.State == "CA")

    df.show()



import timeit
from pyspark.sql import SparkSession



def close_spark_session(spark):
    spark.stop()


def time_operation(operation, spark=None):
    if spark is None:
        spark = create_spark_session()
        elapsed_time = timeit.timeit(lambda: operation(spark), number=1)
        close_spark_session(spark)
    else:
        elapsed_time = timeit.timeit(lambda: operation(spark), number=1)

    return elapsed_time



def main():
    # Cold start timings
    cold_start_op1 = time_operation(test_write)
    cold_start_op2 = time_operation(test_read)

    # Warm start timings
    spark = create_spark_session()
    warm_start_op1 = time_operation(test_write, spark)
    warm_start_op2 = time_operation(test_read, spark)
    close_spark_session(spark)

    print(f"Cold start timings:\nOperation 1: {cold_start_op1:.5f}s\nOperation 2: {cold_start_op2:.5f}s")
    print(f"Warm start timings:\nOperation 1: {warm_start_op1:.5f}s\nOperation 2: {warm_start_op2:.5f}s")

if __name__ == "__main__":
    main()


