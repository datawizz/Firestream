from pyspark.sql import SparkSession
import unittest

from etl_lib.services.spark.client import SparkClient



client = SparkClient(app_name="test", master="local[1]")

# Prepare Data
data = [("Alice", 34), ("Bob", 45), ("Catherine", 29)]
columns = ["Name", "Age"]
df = client.spark_session.createDataFrame(data, columns)

# Write Data
df.write.format("delta").mode("overwrite").save("/tmp/test_delta_lake")

# Read Data
read_df = client.spark_session.read.format("delta").load("/tmp/test_delta_lake")

read_df.show()

# Register DataFrame as table in catalog
read_df.createOrReplaceTempView("people")


client.spark_session.sql("SHOW TABLES IN default").show()


# List all databases
databases = client.spark_session.catalog.listDatabases()
for db in databases:
    print(f"Database name is: {db.name}")

# List all tables in a specific database (e.g., 'default')
tables = client.spark_session.catalog.listTables("default")
for table in tables:
    print(f"Table name is: {table.name}")