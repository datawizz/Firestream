
from etl_lib import SparkClient

def run_test_write_read(spark_client):
    spark_client = spark_client.spark_session
    df = spark_client.createDataFrame([(1, 'foo'), (2, 'bar')], ['id', 'value'])
    write_path = "/tmp/data/temp.parquet"
    df.write.mode("overwrite").parquet(write_path)
    read_df = spark_client.read.parquet(write_path)
    df.show()
    read_df.show()

def main():
    spark_client = SparkClient(app_name="TestSparkClient")
    run_test_write_read(spark_client)

if __name__ == "__main__":
    main()

### Pytest ###

import pytest
@pytest.mark.usefixtures("spark_client")
def test_run_test_write_read(spark_client):
    run_test_write_read(spark_client)
