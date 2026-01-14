import requests
import json

def test_query_spark_sql():
    # Start the Flask app before running the test (e.g., "python app.py")

    # Prepare the test data and register a temporary table in the Spark SQL context
    # You can do this by sending a POST request to an endpoint in your Flask app
    # In this example, let's assume you have a '/register_temp_table' endpoint

    test_data = [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35},
    ]

    response = requests.post("http://localhost:5050/register_temp_table", json={"table_name": "people", "data": test_data})

    if response.status_code != 200:
        print(f"Error: status code {response.status_code}, response content: {response.content}")
        return


    # Test the '/query' endpoint
    test_sql = "SELECT name, age FROM people"
    response = requests.post("http://localhost:5050/query", data={"sql": test_sql})

    assert response.status_code == 200, f"Expected status code 200, but got {response.status_code}"

    result_json = response.json()

    expected_result = [
        {"name": "Alice", "age": 30},
        {"name": "Charlie", "age": 35},
    ]

    assert result_json == expected_result, f"Expected result {expected_result}, but got {result_json}"

    print("Test passed!")


if __name__ == "__main__":
    test_query_spark_sql()
