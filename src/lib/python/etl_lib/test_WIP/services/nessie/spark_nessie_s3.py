from pyspark.sql import SparkSession
import os
os.environ['NESSIE_SERVER_URI'] = "http://nessie.default.svc.cluster.local:19120/api/v1"
os.environ['S3_LOCAL_BUCKET_NAME'] = "fireworks"
os.environ['AWS_PROFILE'] = "default"
os.environ['S3_LOCAL_ENDPOINT_URL'] = "http://minio.default.svc.cluster.local:9000"
os.environ['AWS_ENDPOINT_URL_S3'] = "http://minio.default.svc.cluster.local:9000"
os.environ['S3_LOCAL_DEFAULT_REGION'] = "us-west-2"

spark = SparkSession.builder \
    .appName("IcebergDemo") \
    .config("spark.jars.packages", "org.apache.iceberg:iceberg-spark-runtime-3.2_2.12:0.13.1,software.amazon.awssdk:bundle:2.17.178,software.amazon.awssdk:url-connection-client:2.17.178") \
    .config("spark.sql.extensions", "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions") \
    .config("spark.sql.catalog.nessie", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.nessie.warehouse", F"s3://{os.environ['S3_LOCAL_BUCKET_NAME']}") \
    .config("spark.sql.catalog.nessie.catalog-impl", "org.apache.iceberg.nessie.NessieCatalog") \
    .config("spark.sql.catalog.nessie.io-impl", "org.apache.iceberg.aws.s3.S3FileIO") \
    .config("spark.sql.catalog.nessie.uri", os.environ['NESSIE_SERVER_URI']) \
    .config("spark.sql.catalog.nessie.ref", "main") \
    .config("spark.sql.catalog.nessie.cache-enabled", "false") \
    .config("spark.sql.catalog.nessie.aws.client.endpoint-override", os.environ['S3_LOCAL_ENDPOINT_URL']) \
    .config("spark.sql.catalog.nessie.aws.client.region", os.environ['S3_LOCAL_DEFAULT_REGION']) \
    .config("spark.sql.catalog.nessie.aws.client.path-style-access", "true") \
    .config("spark.sql.catalog.nessie.aws.client.protocol", "http") \
    .config("spark.hadoop.fs.s3a.access.key", os.environ['S3_LOCAL_ACCESS_KEY_ID']) \
    .config("spark.hadoop.fs.s3a.secret.key", os.environ['S3_LOCAL_SECRET_ACCESS_KEY']) \
    .config("spark.hadoop.fs.s3a.endpoint", os.environ['S3_LOCAL_ENDPOINT_URL']) \
    .config("spark.hadoop.fs.s3a.path.style.access", True) \
    .config("spark.hadoop.fs.s3a.impl", "org.apache.hadoop.fs.s3a.S3AFileSystem") \
    .getOrCreate()


spark.sql("USE nessie")


# Create table
spark.sql("CREATE TABLE demo (id bigint, data string)")

# Show tables
spark.sql("SHOW TABLES").show()

# Insert data
spark.sql("INSERT INTO demo (id, data) VALUES (1, 'a')")

# Run a query
spark.sql("SELECT * FROM demo").show()


spark.stop()
