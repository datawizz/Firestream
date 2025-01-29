from etl_lib.services.spark.client import SparkClient
import pytest


@pytest.fixture(scope="session")
def sql_context():
    """
    A Fixture to use the same Spark session (configured with all dependencies)
    for all tests in the PyTest session.
    """

    client = SparkClient()
    session = client.spark_session
    yield session
    session.stop()
