# This script is used to test Kyuubi JDBC connection


from pyspark.sql import SparkSession

# Start Spark session with required jars and extensions
spark = SparkSession.builder \
    .config("spark.jars", "/path/hive-jdbc-x.y.z.jar,/path/kyuubi-extension-spark-jdbc-dialect_-*.jar") \
    .config("spark.sql.extensions", "org.apache.spark.sql.dialect.KyuubiSparkJdbcDialectExtension") \
    .getOrCreate()

# Load data from Kyuubi via HiveDriver as JDBC datasource
jdbcDF = spark.read \
    .format("jdbc") \
    .options(driver="org.apache.hive.jdbc.HiveDriver",
             url="jdbc:hive2://kyuubi_server_ip:port",
             user="user",
             password="password",
             query="select * from testdb.src_table"
             ) \
    .load()

# Create JDBC Datasource table with DDL
spark.sql("""CREATE TABLE kyuubi_table USING JDBC
OPTIONS (
    driver='org.apache.hive.jdbc.HiveDriver',
    url='jdbc:hive2://kyuubi_server_ip:port',
    user='user',
    password='password',
    dbtable='testdb.some_table'
)""")

# Read data to dataframe
jdbcDF = spark.sql("SELECT * FROM kyuubi_table")

# Write data from dataframe in overwrite mode
jdbcDF.writeTo("kyuubi_table").overwrite()

# Write data from query
spark.sql("INSERT INTO kyuubi_table SELECT * FROM some_table")

# Use PySpark with Pandas
import pyspark.pandas as ps

psdf = ps.range(10)
sdf = psdf.to_spark().filter("id > 5")
sdf.show()
