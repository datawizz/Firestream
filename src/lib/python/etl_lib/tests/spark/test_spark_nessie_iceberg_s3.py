

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
        "org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.1",
        "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1",
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

# def create_table(spark):
#     sc = spark.sparkContext
#     jvm = sc._gateway.jvm

#     # import jvm libraries for iceberg catalogs and schemas
#     java_import(jvm, "org.projectnessie.iceberg.NessieCatalog")
#     java_import(jvm, "org.apache.iceberg.catalog.TableIdentifier")
#     java_import(jvm, "org.apache.iceberg.Schema")
#     java_import(jvm, "org.apache.iceberg.types.Types")
#     java_import(jvm, "org.apache.iceberg.PartitionSpec")

#     # first instantiate the catalog
#     catalog = jvm.NessieCatalog()
#     catalog.setConf(sc._jsc.hadoopConfiguration())
#     # other catalog properties can be added based on the requirement. For example, "io-impl","authentication.type", etc.
#     catalog.initialize("nessie", {"ref": _REF,
#         "uri": _SERVER,
#         "warehouse": f"{bucket}/spark_warehouse/iceberg"})

#     # Creating table by first creating a table name with namespace
#     region_name = jvm.TableIdentifier.parse("testing.region")

#     # next create the schema
#     region_schema = jvm.Schema([
#     jvm.Types.NestedField.optional(
#         1, "R_REGIONKEY", jvm.Types.LongType.get()
#     ),
#     jvm.Types.NestedField.optional(
#         2, "R_NAME", jvm.Types.StringType.get()
#     ),
#     jvm.Types.NestedField.optional(
#         3, "R_COMMENT", jvm.Types.StringType.get()
#     ),
#     ])

#     # and the partition
#     region_spec = jvm.PartitionSpec.unpartitioned()

#     # finally create the table
#     region_table = catalog.createTable(region_name, region_schema, region_spec)





# def test_write(spark):

#     sc = spark.sparkContext
#     spark.sparkContext.setLogLevel("WARN")
#     # print(f"Hadoop version = {sc._jvm.org.apache.hadoop.util.VersionInfo.getVersion()}")

#     # Read the known good data
#     #TODO make this file read relative to the files instead of the fully qualified domain name
#     df = spark.read.csv(_DATA, header=True, inferSchema=True)

#     df.show()

#     #df.write.format("parquet").mode("overwrite").save(_PATH)

#     df.write.format("iceberg").mode("overwrite").save("dev_catalog.testing.cities")





# def test_read(spark):

#     df = spark.read.parquet(_PATH)

#     df.show()
#     df.printSchema()

#     df = df.select("State").where(df.State == "CA")

#     df.show()



# import timeit
# from pyspark.sql import SparkSession



# def close_spark_session(spark):
#     spark.stop()


# def time_operation(operation, spark=None):
#     if spark is None:
#         spark = create_spark_session()
#         elapsed_time = timeit.timeit(lambda: operation(spark), number=1)
#         close_spark_session(spark)
#     else:
#         elapsed_time = timeit.timeit(lambda: operation(spark), number=1)

#     return elapsed_time



# def main():
#     # Cold start timings
#     cold_start_op1 = time_operation(test_write)
#     cold_start_op2 = time_operation(test_read)

#     # Warm start timings
#     spark = create_spark_session()
#     warm_start_op1 = time_operation(test_write, spark)
#     warm_start_op2 = time_operation(test_read, spark)
#     close_spark_session(spark)

#     print(f"Cold start timings:\nOperation 1: {cold_start_op1:.5f}s\nOperation 2: {cold_start_op2:.5f}s")
#     print(f"Warm start timings:\nOperation 1: {warm_start_op1:.5f}s\nOperation 2: {warm_start_op2:.5f}s")

# if __name__ == "__main__":

#     spark = create_spark_session()
#     spark.sql(f"CREATE BRANCH IF NOT EXISTS {_REF} IN {_CATALOG}").toPandas()
#     spark.sql(f"USE REFERENCE {_REF} IN {_CATALOG}")
#     spark.sql(
#     f"""CREATE TABLE IF NOT EXISTS {_CATALOG}.salesdip.sales
#             (id STRING, name STRING, product STRING, price STRING, date STRING) USING iceberg"""
#     ).collect()
#     spark.sql(f"SELECT * FROM {_CATALOG}.salesdip.sales").show()
#     spark.sql("SHOW TABLES IN salesdip").show()


# def test_pipeline_extended():

#     spark = create_spark_session()

#     # Use catalog
#     spark.sql(F"USE {_CATALOG}").show()

#     # Create branch if not exists
#     spark.sql(F"CREATE BRANCH IF NOT EXISTS testing IN {_CATALOG}").show()
#     spark.sql(F"USE REFERENCE testing IN {_CATALOG}").show()

#     # Drop table if exists
#     spark.sql("DROP TABLE IF EXISTS demo").show()

#     # Create namespace if not exists
#     spark.sql("CREATE NAMESPACE IF NOT EXISTS testNamespace").show()

#     # Create table in new namespace
#     spark.sql("CREATE TABLE IF NOT EXISTS testNamespace.demo (id bigint, data string) USING iceberg").show()

#     # Show tables in testNamespace
#     spark.sql("SHOW TABLES IN testNamespace").show()

#     # Insert data
#     spark.sql("INSERT INTO testNamespace.demo (id, data) VALUES (1, 'a'), (2, 'b')").show()

#     # Run a query
#     spark.sql("SELECT * FROM testNamespace.demo").show()

#     # Merge branch 'testing' to 'main'
#     spark.sql(F"MERGE BRANCH testing TO main IN {_CATALOG}").show()

#     # Switch to 'main' branch
#     spark.sql(F"USE REFERENCE main IN {_CATALOG}").show()

#     # Read the data in 'main'
#     spark.sql("SELECT * FROM testNamespace.demo").show()

#     spark.stop()