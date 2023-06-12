import json
import os
from pyspark.sql import SparkSession
from pyspark.sql.types import StringType

def merge_schemas(schema1, schema2):
    return schema1.merge(schema2)

def main():
    # Initialize the Spark session
    spark = SparkSession.builder \
        .appName("Infer Common JSON Schema") \
        .getOrCreate()

    # Replace this with the path to your JSON files directory
    json_files_directory = "/workspace/submodules/quadratichq/quadratic/public/examples/"

    # Initialize an empty schema
    common_schema = None

    # Iterate through JSON files in the directory
    for file in os.listdir(json_files_directory):
        if file.endswith(".grid"):
            json_file_path = os.path.join(json_files_directory, file)

            # Read the entire JSON content into a single record
            rdd = spark.sparkContext.wholeTextFiles(json_file_path)

            # Parse the JSON content using Python's json module
            parsed_json = rdd.map(lambda x: json.loads(x[1]))

            # Convert the RDD to a DataFrame
            json_str = parsed_json.map(lambda x: json.dumps(x))
            json_df = json_str.toDF(StringType()).selectExpr("value as json")

            # Infer the schema using the from_json function
            df = spark.read.json(json_df.rdd.map(lambda x: x.json))

            df.printSchema()
            
if __name__ == "__main__":
    main()
