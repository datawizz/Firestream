import pytest

@pytest.mark.usefixtures("spark_client")
class TestSparkApp:
    def test_data_count(self, spark_client):
        self.spark_client = spark_client
        data = [("Alice", 34), ("Bob", 45), ("Catherine", 29)]
        df = self.spark_client.spark_session.createDataFrame(data, ["Name", "Age"])
        assert df.count() == 3

    def test_column_names(self, spark_client):
        self.spark_client = spark_client
        data = [("Alice", 34), ("Bob", 45), ("Catherine", 29)]
        df = self.spark_client.spark_session.createDataFrame(data, ["Name", "Age"])
        assert df.columns == ["Name", "Age"]

