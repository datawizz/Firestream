from sqlalchemy import create_engine

# Define JDBC connection parameters
jdbc_driver = "org.apache.hive.jdbc.HiveDriver"
jdbc_url = "jdbc:hive2://localhost:10000/my_database"
jdbc_user = "my_user"
jdbc_password = "my_password"

# Create the SQLAlchemy engine
engine = create_engine(f"jdbc:{jdbc_url}", connect_args={"user": jdbc_user, "password": jdbc_password}, driver_class=jdbc_driver)

# Define a Delta table name and path
table_name = "my_delta_table"
table_path = "s3a://my-bucket/my-delta-table"

# Create a Delta table using PySpark
from pyspark.sql import SparkSession
spark = SparkSession.builder.appName("Create Delta Table").getOrCreate()
data = [("John", "Doe"), ("Jane", "Doe")]
df = spark.createDataFrame(data, ["first_name", "last_name"])
df.write.format("delta").mode("overwrite").save(table_path)

# Query the Delta table using SQLAlchemy
with engine.connect() as conn:
    results = conn.execute(f"SELECT * FROM {table_name}")
    for row in results:
        print(row)

# Insert new data into the Delta table using SQLAlchemy
new_data = [("Bob", "Smith"), ("Alice", "Smith")]
values_str = ",".join([f"('{fn}', '{ln}')" for fn, ln in new_data])
with engine.connect() as conn:
    conn.execute(f"INSERT INTO {table_name} VALUES {values_str}")

# Close the SQLAlchemy engine
engine.dispose()
