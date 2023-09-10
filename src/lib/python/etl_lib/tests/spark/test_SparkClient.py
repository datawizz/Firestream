# import os
# import pytest
# from pyspark.sql import DataFrame
# from etl_lib import SparkClient
# import os
# import pytest


# @pytest.fixture(scope="module", params=["local"])
# def spark_client(request):
#     app_name = "TestApp"
#     config = {"key": "value"}
#     storage_location = request.param
#     client = SparkClient(app_name=app_name, config=config, storage_location=storage_location)
#     yield client
#     client.session.stop()

# @pytest.fixture
# def test_df(spark_client):
#     test_data = [("Alice", 25), ("Bob", 30)]
#     return spark_client.session.createDataFrame(test_data, ["Name", "Age"])

# # def compare_dataframes(df1: DataFrame, df2: DataFrame) -> bool:
# #     return df1.subtract(df2).count() == 0 and df2.subtract(df1).count() == 0

# def test_set_bucket(spark_client):
#     spark_client.set_bucket()
#     expected_url = os.environ.get(f"S3_{spark_client.storage_location.upper()}_ENDPOINT_URL")
#     assert spark_client.S3_ENDPOINT_URL == expected_url

# # def test_config_spark(spark_client):
# #     spark_client.config_spark()
# #     assert spark_client.config.get("spark.hadoop.fs.s3a.endpoint") == spark_client.S3_ENDPOINT_URL

# # def test_create_spark_session(spark_client):
# #     session = spark_client.create_spark_session()
# #     assert session is not None
# #     assert session.sparkContext.getConf().get("spark.app.name") == spark_client.app_name

# # @pytest.mark.parametrize("catalog", ["nessie", "delta"])
# # def test_integration_with_catalogs(spark_client, test_df, catalog):
# #     spark_client.CATALOG = catalog
# #     test_table = f"{spark_client.CATALOG}.test_table"
# #     test_df.write.mode('overwrite').saveAsTable(test_table)
# #     read_df = spark_client.session.sql(f"SELECT * FROM {test_table}")
# #     assert compare_dataframes(test_df, read_df)



# import os
# import unittest
# from pyspark.sql import DataFrame
# from etl_lib import SparkClient

# class TestSparkClient(unittest.TestCase):
    
#     @classmethod
#     def setUpClass(cls):
#         app_name = "TestApp"
#         config = {"key": "value"}
#         storage_location = "local"
#         cls.spark = SparkClient(app_name=app_name, config=config, storage_location=storage_location)

#     @classmethod
#     def tearDownClass(cls):
#         cls.spark.session.stop()

#     def setUp(self):
#         self.test_data = [("Alice", 25), ("Bob", 30)]
#         self.test_df = self.spark.session.createDataFrame(self.test_data, ["Name", "Age"])

#     def test_set_bucket(self):
#         self.spark.set_bucket()
#         expected_url = os.environ.get(f"S3_{self.spark.storage_location.upper()}_ENDPOINT_URL")
#         self.assertEqual(self.spark.S3_ENDPOINT_URL, expected_url)

#     def test_write_read(self):
#         self.df = self.spark.session.createDataFrame([(1, 'foo'), (2, 'bar')], ['id', 'value'])
#         write_path = "/tmp/data/temp.parquet"
#         self.spark.write(self.df, "parquet", path=write_path)
#         read_df = self.spark.read("parquet", path=write_path, mode="overwrite")
#         self.df.show()
#         read_df.show()
#         # self.assertEqual(
#         #     sorted(self.df.collect(), key=lambda row: tuple(row)),
#         #     sorted(read_df.collect(), key=lambda row: tuple(row))
#         # )

#     def tearDown(self):
#         pass


# if __name__ == "__main__":
#     import pytest
#     pytest.main([__file__])

from etl_lib import SparkClient


def test_write_read(spark_client):
    df = spark_client.spark_session.createDataFrame([(1, 'foo'), (2, 'bar')], ['id', 'value'])
    write_path = "/tmp/data/temp.parquet"
    spark_client.write(df, "parquet", path=write_path)
    read_df = spark_client.read("parquet", path=write_path, mode="overwrite")
    df.show()
    read_df.show()
    # assert (
    #     sorted(df.collect(), key=lambda row: tuple(row)) ==
    #     sorted(read_df.collect(), key=lambda row: tuple(row))
    # )




import os
import pytest


@pytest.mark.usefixtures("spark_client")
class TestSparkClient:
    
    @pytest.fixture(autouse=True)
    def setUp(self, spark_client):
        self.spark_client = spark_client
        self.test_data = [("Alice", 25), ("Bob", 30)]
        
        self.test_df = self.spark_client.spark_session.createDataFrame(data=self.test_data, schema=["Name", "Age"])



if __name__ == "__main__":

    client = SparkClient(app_name="TestApp")
    test_write_read(client)
    