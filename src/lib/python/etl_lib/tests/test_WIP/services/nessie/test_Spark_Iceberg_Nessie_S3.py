from pyspark.sql import SparkSession
from pyspark.sql.functions import *

# Full url of the Nessie API endpoint to nessie
url = "http://localhost:19120/api/v1"
# Where to store nessie tables
full_path_to_warehouse = "s3a://your_bucket/your_directory"  # Update it to your S3 path
# The ref or context that nessie will operate on (if different from default branch).
# Can be the name of a Nessie branch or tag name.
ref = "main"
# Nessie authentication type (BASIC, NONE or AWS)
auth_type = "NONE"

# here we are assuming NONE authorisation
spark = SparkSession.builder \
        .config("spark.jars.packages","org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.0,org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1") \
        .config("spark.sql.extensions", "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions") \
        .config("spark.sql.catalog.nessie.uri", url) \
        .config("spark.sql.catalog.nessie.ref", ref) \
        .config("spark.sql.catalog.nessie.authentication.type", auth_type) \
        .config("spark.sql.catalog.nessie.catalog-impl", "org.apache.iceberg.nessie.NessieCatalog") \
        .config("spark.sql.catalog.nessie.warehouse", full_path_to_warehouse) \
        .config("spark.sql.catalog.nessie", "org.apache.iceberg.spark.SparkCatalog") \
        .getOrCreate()

# Generating some data
data = [("Alice", 1), ("Bob", 2), ("Charlie", 3)]
df = spark.createDataFrame(data, ["Name", "ID"])

# Writing data to S3 in iceberg format using nessie catalog
df.write.format("iceberg").mode("overwrite").saveAsTable("nessie.testing.iceberg_table")

# Reading the data back
df_read = spark.read.format("iceberg").load("nessie.testing.iceberg_table")
df_read.show()
