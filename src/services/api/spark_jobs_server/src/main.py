# Implements a REST API for the ETL library
# Allows for SQL query execution and data retrieval via persistant Spark Context

from flask import Flask, request, jsonify
from pyspark.sql import SparkSession

import logging

logging.basicConfig(level=logging.DEBUG, format='%(asctime)s [%(levelname)s] %(message)s')


app = Flask(__name__)

# Initialize the Spark session with a unique app name
spark = SparkSession.builder \
    .appName("Flask-Spark-SQL-App") \
    .getOrCreate()

# Create the persistent Spark SQL context
sqlContext = spark.newSession()


@app.route('/query', methods=['POST'])
def query_spark_sql():
    if request.method == 'POST':
        sql = request.form.get('sql')

        if sql is None or sql.strip() == '':
            return jsonify({"error": "Please provide a valid SQL query"}), 400

        try:
            # Execute the SQL query and collect the results into an RDD
            result = sqlContext.sql(sql)

            # Convert the RDD to JSON
            result_json = result.toJSON().collect()

            # Return the JSON representation of the RDD
            print(result)
            return jsonify(result_json), 200

        except Exception as e:
            logging.exception("An error occurred while executing the SQL query:")
            return jsonify({"error": str(e)}), 500

@app.route('/register_temp_table', methods=['POST'])
def register_temp_table():
    if request.method == 'POST':
        table_name = request.json.get('table_name')
        data = request.json.get('data')

        if table_name is None or table_name.strip() == '':
            return jsonify({"error": "Please provide a valid table name"}), 400

        if data is None or not isinstance(data, list):
            return jsonify({"error": "Please provide a valid data list"}), 400

        try:
            # Convert the input data to a Spark DataFrame
            df = spark.createDataFrame(data)

            # Register the DataFrame as a temporary table
            df.createOrReplaceTempView(table_name)

            logging.info(f"Temporary table '{table_name}' registered successfully")


            return jsonify({"message": f"Temporary table '{table_name}' registered"}), 200

        # For the /register_temp_table endpoint
        except Exception as e:
            logging.exception("An error occurred while registering the temporary table:")
            return jsonify({"error": str(e)}), 500


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5050, debug=True)
