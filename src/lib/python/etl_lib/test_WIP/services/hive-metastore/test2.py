from hive_metastore_client import HiveMetastoreClient
from hive_metastore_client.models.table import Table

# Connect to Hive Metastore
metastore_uri = "thrift://localhost:9083"
client = HiveMetastoreClient(metastore_uri)

# Create a new table
def create_table(database, table_name, columns, partition_keys=None):
    new_table = Table(
        db_name=database,
        table_name=table_name,
        cols=columns,
        partition_keys=partition_keys
    )
    client.create_table(new_table)

# Read table metadata
def read_table(database, table_name):
    table = client.get_table(database, table_name)
    return table

# Update table metadata
def update_table(database, table_name, new_columns, new_partition_keys=None):
    table = client.get_table(database, table_name)
    table.cols = new_columns
    if new_partition_keys is not None:
        table.partition_keys = new_partition_keys
    client.alter_table(database, table_name, table)

# Delete a table
def delete_table(database, table_name):
    client.drop_table(database, table_name)

# Example usage
database = "my_database"
table_name = "my_table"

# Column and partition key definitions
columns = [
    {"name": "id", "type": "bigint", "comment": "ID"},
    {"name": "name", "type": "string", "comment": "Name"},
]

partition_keys = [
    {"name": "year", "type": "int", "comment": "Year"},
    {"name": "month", "type": "int", "comment": "Month"},
]

# Create a table
create_table(database, table_name, columns, partition_keys)

# Read table metadata
table = read_table(database, table_name)
print(table)

# Update table metadata
new_columns = [
    {"name": "id", "type": "bigint", "comment": "ID"},
    {"name": "first_name", "type": "string", "comment": "First Name"},
    {"name": "last_name", "type": "string", "comment": "Last Name"},
]

update_table(database, table_name, new_columns)

# Delete table
delete_table(database, table_name)
