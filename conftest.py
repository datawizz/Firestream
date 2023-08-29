import pytest
from etl_lib.services.spark.client import SparkClient

@pytest.fixture(scope='session')
def spark_client():
    """
    Create a SparkClient for testing
    """

    client = SparkClient(app_name="Pytest", config={}, storage_location="local")
    yield client

    client.stop()

@pytest.fixture(scope='session')
def spark_context(spark_client):
    """
    Return a SparkContext for testing (aka "spark")
    """

    spark = spark_client.spark_context

    yield spark

    # spark.session.stop()


@pytest.fixture(scope='session')
def spark_session(spark_context):
    """
    Return a SparkSession
    """

    yield spark_context.session

    # spark_context.stop(), hanlded in spark_context fixture