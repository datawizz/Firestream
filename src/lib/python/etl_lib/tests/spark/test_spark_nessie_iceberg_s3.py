

from pyspark.sql import *
from pyspark.sql import SparkSession
import os

from etl_lib.services.spark.client import SparkClient
import timeit

_BUCKET = os.environ.get("S3_LOCAL_BUCKET_NAME")
_SERVER = os.environ.get("NESSIE_SERVER_URI")
_S3_LOCAL_ENDPOINT_URL = os.environ.get("S3_LOCAL_ENDPOINT_URL")
_S3_LOCAL_ACCESS_KEY_ID = os.environ.get("S3_LOCAL_ACCESS_KEY_ID")
_S3_LOCAL_SECRET_ACCESS_KEY = os.environ.get("S3_LOCAL_SECRET_ACCESS_KEY")
_PATH = f"s3a://{_BUCKET}/spark_warehouse"
_LOG_PATH = f"s3a://{_BUCKET}/spark_logs"
os.environ["SPARK_CLASSPATH"] = "/workspace/target/dependency"
_LOG_DIR = "spark_logs/"
_REF = "main"
_CATALOG = "blahblahblah"
_NESSIE_VERSION = os.environ.get("NESSIE_VERSION")

#TODO port this to a data source


def create_logging_dir():
    """
    Make it idempotent
    """

    import boto3
    import os

    s3 = boto3.resource(
        's3',
        endpoint_url=os.environ['S3_LOCAL_ENDPOINT_URL'],
        aws_access_key_id=os.environ['S3_LOCAL_ACCESS_KEY_ID'],
        aws_secret_access_key=os.environ['S3_LOCAL_SECRET_ACCESS_KEY'],
        region_name=os.environ['S3_LOCAL_DEFAULT_REGION']
    )

    bucket = s3.Bucket(os.environ['S3_LOCAL_BUCKET_NAME'])
    dir_obj = None

    try:
        s3.meta.client.head_object(Bucket=os.environ['S3_LOCAL_BUCKET_NAME'], Key=_LOG_DIR)
        dir_obj = bucket.Object(key=_LOG_DIR)
    except Exception as e:
        print("Directory does not exist, creating it now.")
        
    if dir_obj is None:
        bucket.put_object(Key=_LOG_DIR)


create_logging_dir()



def create_spark_session():

    jars_packages = [
        # For Kafka access
        "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1",
        "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1",
        "org.apache.kafka:kafka-clients:3.3.1",
        "org.apache.iceberg:iceberg-spark-runtime-3.4_2.12:1.3.0",
        "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.4_2.12:0.70.0",
        "org.apache.hadoop:hadoop-aws:3.3.1",
        "org.apache.hadoop:hadoop-common:3.3.1",
        "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1"
    ]
    jars_packages = ",".join(jars_packages)

    config = {
        "spark.hadoop.fs.s3a.endpoint": _S3_LOCAL_ENDPOINT_URL,
        "spark.hadoop.fs.s3a.connection.ssl.enabled": "false",
        "spark.hadoop.fs.s3a.path.style.access": "true",
        "spark.hadoop.fs.s3a.committer.name": "directory",
        "spark.hadoop.fs.s3a.aws.credentials.provider": "org.apache.hadoop.fs.s3a.SimpleAWSCredentialsProvider",
        "spark.hadoop.fs.s3a.access.key": _S3_LOCAL_ACCESS_KEY_ID,
        "spark.hadoop.fs.s3a.secret.key": _S3_LOCAL_SECRET_ACCESS_KEY,
        "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
        "spark.sql.execution.arrow.pyspark.enabled": "true",
        "spark.sql.execution.arrow.pyspark.enabled": "true",
        "spark.sql.session.timeZone": "UTC",
        "spark.sql.streaming.checkpointLocation": "/tmp/spark_checkpoint", # TODO when deployed to K8 does this persist?
        F"spark.sql.catalog.{_CATALOG}.warehouse": _PATH,
        F"spark.sql.catalog.{_CATALOG}": "org.apache.iceberg.spark.SparkCatalog",
        F"spark.sql.catalog.{_CATALOG}.catalog-impl": "org.apache.iceberg.nessie.NessieCatalog",
        F"spark.sql.catalog.{_CATALOG}.uri": _SERVER,
        F"spark.sql.catalog.{_CATALOG}.ref": _REF,
        F"spark.sql.catalog.{_CATALOG}.auth_type": "NONE",
        "spark.sql.extensions": "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions",
        "spark.jars.packages": jars_packages,
        # Set logging to use S3
        "spark.eventLog.enabled": "true",
        "spark.eventLog.dir": _LOG_PATH,
        # "spark.history.fs.logDirectory": F"s3a://{_BUCKET}/",
        "spark.eventLog.rolling.enabled": "true",
        "spark.eventLog.rolling.maxFileSize": "128m",
        # "spark.driver.extraClassPath": "/workspace/target/dependency/*",
        # "spark.executor.extraClassPath": "/workspace/target/dependency/*"
    }


    app_name = "test"

    spark = SparkSession.builder.master("local[1]").appName(app_name)

    for key, value in config.items():
        spark.config(key, value)

    return spark.getOrCreate()







def test_extended_pipeline():
    spark = create_spark_session()

    # Using the configured catalog
    spark.sql(F"USE {_CATALOG}").show()

    # Create a new namespace
    spark.sql(F"CREATE NAMESPACE IF NOT EXISTS {_CATALOG}.namespace_test").show()

    # Create the 'testing' branch
    spark.sql(F"CREATE BRANCH IF NOT EXISTS testing IN {_CATALOG}").show()


    # Using the newly created namespace
    spark.sql(F"USE {_CATALOG}.namespace_test").show()
    spark.sql(F"USE REFERENCE testing IN {_CATALOG}")

    # Create tables in the namespace
    spark.sql("CREATE TABLE IF NOT EXISTS table_one (id bigint, data string) USING iceberg").show()
    spark.sql("CREATE TABLE IF NOT EXISTS other_table (id bigint, data string) USING iceberg").show()

    # Show the tables
    spark.sql("SHOW TABLES").show()
    # spark.sql("DESCRIBE TABLE table_one").show()
    
    # Start transaction and insert data into both tables
    spark.sql("INSERT INTO table_one (id, data) VALUES (1, 'data1')").show()
    spark.sql("INSERT INTO table_one (id, data) VALUES (3, 'data3')").show()
    spark.sql("INSERT INTO other_table (id, data) VALUES (2, 'data2')").show()

    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()

    # Merge 'testing' reference to 'main'
    spark.sql(F"MERGE BRANCH testing INTO main IN {_CATALOG}").show()


    # Switch to 'main' reference and verify the data
    spark.sql(F"USE REFERENCE main IN {_CATALOG}").show()
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()

    # Clean up
    spark.sql(F"DROP BRANCH testing IN {_CATALOG}").show()
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.table_one").show()
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.other_table").show()

    spark.stop()


def test_merge_into():
    spark = create_spark_session()

    # Using the configured catalog
    spark.sql(F"USE {_CATALOG}").show()

    # Create a new namespace
    spark.sql(F"CREATE NAMESPACE IF NOT EXISTS {_CATALOG}.namespace_test").show()

    # Create the 'testing' branch
    spark.sql(F"CREATE BRANCH IF NOT EXISTS testing IN {_CATALOG}").show()

    # Using the newly created namespace
    spark.sql(F"USE {_CATALOG}.namespace_test").show()
    spark.sql(F"USE REFERENCE testing IN {_CATALOG}")

    # Create tables in the namespace
    spark.sql("CREATE TABLE IF NOT EXISTS table_one (id bigint, data string) USING iceberg").show()
    spark.sql("CREATE TABLE IF NOT EXISTS other_table (id bigint, data string) USING iceberg").show()

    # Start transaction and insert data into both tables
    spark.sql("INSERT INTO table_one (id, data) VALUES (1, 'data1')").show()
    spark.sql("INSERT INTO table_one (id, data) VALUES (3, 'data3')").show()
    spark.sql("INSERT INTO other_table (id, data) VALUES (2, 'data2')").show()

    # Create a temporary view with new data
    spark.sql("""
        CREATE OR REPLACE TEMPORARY VIEW source_table AS 
        SELECT * FROM (
            VALUES 
                (1, 'updated_data1'),
                (2, 'updated_data2'),
                (3, 'updated_data3'),
                (4, 'data4')
        ) AS source_table(id, data)
    """)

    # Merge the data from the temporary view into the tables
    spark.sql("""
        MERGE INTO table_one
        USING source_table
        ON table_one.id = source_table.id
        WHEN MATCHED THEN UPDATE SET table_one.data = source_table.data
        WHEN NOT MATCHED THEN INSERT (id, data) VALUES (source_table.id, source_table.data)
    """).show()

    spark.sql("""
        MERGE INTO other_table
        USING source_table
        ON other_table.id = source_table.id
        WHEN MATCHED THEN UPDATE SET other_table.data = source_table.data
        WHEN NOT MATCHED THEN INSERT (id, data) VALUES (source_table.id, source_table.data)
    """).show()

    # Continue with the rest of your code...
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()

    # Merge 'testing' reference to 'main'
    spark.sql(F"MERGE BRANCH testing INTO main IN {_CATALOG}").show()

    # Switch to 'main' reference and verify the data
    spark.sql(F"USE REFERENCE main IN {_CATALOG}").show()
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()

    # Clean up
    spark.sql(F"DROP BRANCH testing IN {_CATALOG}").show()
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.table_one").show()
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.other_table").show()

    spark.stop()



def test_extended_pipeline2():
    spark = create_spark_session()

    # Using the configured catalog
    start = timeit.default_timer()
    spark.sql(F"USE {_CATALOG}")
    end = timeit.default_timer()
    print(f'USE catalog took: {end - start} seconds')

    # Create a new namespace
    start = timeit.default_timer()
    spark.sql(F"CREATE NAMESPACE IF NOT EXISTS {_CATALOG}.namespace_test")
    end = timeit.default_timer()
    print(f'Create namespace took: {end - start} seconds')

    # Create the 'testing' branch
    start = timeit.default_timer()
    spark.sql(F"CREATE BRANCH IF NOT EXISTS testing IN {_CATALOG}")
    end = timeit.default_timer()
    print(f'Create branch took: {end - start} seconds')

    # Using the newly created namespace
    start = timeit.default_timer()
    spark.sql(F"USE {_CATALOG}.namespace_test")
    spark.sql(F"USE REFERENCE testing IN {_CATALOG}")
    end = timeit.default_timer()
    print(f'Use namespace and branch took: {end - start} seconds')

    # Create tables in the namespace
    start = timeit.default_timer()
    spark.sql("CREATE TABLE IF NOT EXISTS table_one (id bigint, data string) USING iceberg")
    spark.sql("CREATE TABLE IF NOT EXISTS other_table (id bigint, data string) USING iceberg")
    end = timeit.default_timer()
    print(f'Create tables took: {end - start} seconds')



    # Start transaction and insert data into both tables
    start = timeit.default_timer()
    spark.sql("INSERT INTO table_one (id, data) VALUES (1, 'data1')")
    spark.sql("INSERT INTO table_one (id, data) VALUES (3, 'data3')")
    spark.sql("INSERT INTO other_table (id, data) VALUES (2, 'data2')")
    end = timeit.default_timer()
    print(f'Insert data took: {end - start} seconds')

    # Cache the tables
    spark.catalog.cacheTable("table_one")
    spark.catalog.cacheTable("other_table")

    # Query tables (cached)
    start = timeit.default_timer()
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()
    end = timeit.default_timer()
    print(f'Query cached tables took: {end - start} seconds')

    # Uncache the tables
    spark.catalog.uncacheTable("table_one")
    spark.catalog.uncacheTable("other_table")

    # Query tables (uncached)
    start = timeit.default_timer()
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()
    end = timeit.default_timer()
    print(f'Query uncached tables took: {end - start} seconds')

    # Merge 'testing' reference to 'main'
    start = timeit.default_timer()
    spark.sql(F"MERGE BRANCH testing INTO main IN {_CATALOG}")
    end = timeit.default_timer()
    print(f'Merge branch took: {end - start} seconds')

    # Switch to 'main' reference and verify the data
    start = timeit.default_timer()
    spark.sql(F"USE REFERENCE main IN {_CATALOG}")
    spark.sql("SELECT * FROM table_one").show()
    spark.sql("SELECT * FROM other_table").show()
    end = timeit.default_timer()
    print(f'Switch to main reference and query tables took: {end - start} seconds')

    # Clean up
    start = timeit.default_timer()
    spark.sql(F"DROP BRANCH testing IN {_CATALOG}")
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.table_one")
    spark.sql(F"DROP TABLE {_CATALOG}.namespace_test.other_table")
    end = timeit.default_timer()
    print(f'Clean up took: {end - start} seconds')

    spark.stop()


if __name__ == "__main__":
    test_extended_pipeline()
    test_merge_into()

    test_extended_pipeline2()
