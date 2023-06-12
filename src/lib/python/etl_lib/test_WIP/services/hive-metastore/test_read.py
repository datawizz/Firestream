from hive_metastore_client import HiveMetastoreClient
import os

METASTORE_URL = os.environ.get('METASTORE_URL')
METASTORE_PORT = os.environ.get('METASTORE_PORT')

# Create a client
client = HiveMetastoreClient(METASTORE_URL, METASTORE_PORT)

# List all databases
databases = client.get_all_databases()
print("Databases:", databases)

# List tables in a specific database
database_name = "default"
tables = client.get_all_tables(database_name)
print(f"Tables in {database_name}:", tables)

# Get table details
table_name = "sample_table"
table = client.get_table(database_name, table_name)
print(f"Table {table_name} details:", table)

# Close the client connection
client.close()
