#!/bin/bash

# Define Environment Variables for Spark
export DBT_SPARK_SCHEMA=my_spark_database
export DBT_SPARK_HOST=localhost
export DBT_SPARK_PORT=10000
export DBT_SPARK_USER=spark_user
export DBT_SPARK_PASSWORD=spark_password

# Define Environment Variables for PostgreSQL
export DBT_PG_SCHEMA=postgres
export DBT_PG_HOST=THIS_IS_A_SECRET_TODO_CHANGE_ME
export DBT_PG_PORT=5432
export DBT_PG_USER=pguser
export DBT_PG_PASSWORD=THIS_IS_A_SECRET_TODO_CHANGE_ME

# Path to the DBT profiles directory
DBT_PROFILES_DIR=~/.dbt

# Ensure the directory exists
mkdir -p $DBT_PROFILES_DIR

# Create the profiles.yml file
cat > $DBT_PROFILES_DIR/profiles.yml << EOF
spark_connection:
  target: dev
  outputs:
    dev:
      type: spark
      method: thrift
      schema: $DBT_SPARK_SCHEMA
      host: $DBT_SPARK_HOST
      port: $DBT_SPARK_PORT
      user: $DBT_SPARK_USER
      password: $DBT_SPARK_PASSWORD
      connect_timeout: 10
      connect_retries: 2
      threads: 4

pg_connection:
  target: dev
  outputs:
    dev:
      type: postgres
      schema: $DBT_PG_SCHEMA
      host: $DBT_PG_HOST
      port: $DBT_PG_PORT
      user: $DBT_PG_USER
      pass: $DBT_PG_PASSWORD
      threads: 4
      keepalives_idle: 0
EOF

echo "DBT profiles.yml file created successfully."
