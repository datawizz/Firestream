


import json
from pyspark.sql import SparkSession
from pyspark.sql.types import StringType
from pyspark.sql.functions import from_json

def main():
    # Initialize the Spark session
    spark = SparkSession.builder \
        .appName("Infer JSON Schema") \
        .getOrCreate()

    # Replace this with the path to your JSON file
    json_file_path = "/workspace/src/lib/python/etl_lib/etl_lib/services/quadratic/example.json"

    # Read the entire JSON content into a single record
    rdd = spark.sparkContext.wholeTextFiles(json_file_path)

    # Parse the JSON content using Python's json module
    parsed_json = rdd.map(lambda x: json.loads(x[1]))

    # Convert the RDD to a DataFrame
    json_str = parsed_json.map(lambda x: json.dumps(x))
    json_df = json_str.toDF(StringType()).selectExpr("value as json")

    # Infer the schema using the from_json function
    df = spark.read.json(json_df.rdd.map(lambda x: x.json))

    # Print the schema
    df.printSchema()

if __name__ == "__main__":
    main()
