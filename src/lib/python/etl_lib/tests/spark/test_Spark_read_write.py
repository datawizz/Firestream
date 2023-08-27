# import pytest
# from pyspark.sql import SparkSession
# from etl_lib import SparkClient
# from pyspark.sql.types import StructType, StructField, StringType

# # Test fixture for SparkSession
# @pytest.fixture(scope="session")
# def spark_client(request):
#     app_name = "TestApp"
#     config = {"key": "value"}
#     client = SparkClient(app_name=app_name, config=config, storage_location="local")
#     yield client
#     client.session.stop()


# # Test case for read method
# def test_read(spark_client, tmp_path):
#     data = [("Alice",), ("Bob",)]
#     schema = StructType([StructField("name", StringType(), True)])
#     df = spark_client.session.createDataFrame(data, schema=schema)
#     path = tmp_path / "read_test.parquet"
#     df.write.parquet(str(path))
#     read_df = spark_client.read("parquet", path=str(path))
#     # assert read_df.count() == 2

# # Test case for write method
# def test_write(spark_client, tmp_path):
#     data = [("Alice",), ("Bob",)]
#     schema = StructType([StructField("name", StringType(), True)])
#     df = spark_client.session.createDataFrame(data, schema=schema)
#     path = tmp_path / "write_test.parquet"
#     spark_client.write(df, "parquet", path=str(path))
#     read_df = spark_client.session.read.parquet(str(path))
#     assert read_df.count() == 2

# # Test case for read_stream method
# def test_read_stream(spark_client, tmp_path):
#     data = [("Alice",), ("Bob",)]
#     schema = StructType([StructField("name", StringType(), True)])
#     df = spark_client.session.createDataFrame(data, schema=schema)
#     path = tmp_path / "stream_test.parquet"
#     df.write.parquet(str(path))
#     read_stream_df = spark_client.read_stream("parquet", path=str(path))
#     query = read_stream_df.writeStream.outputMode("append").format("memory").queryName("test_stream").start()
#     query.processAllAvailable()
#     result = spark_client.session.sql("SELECT * FROM test_stream")
#     assert result.count() == 2
#     query.stop()
