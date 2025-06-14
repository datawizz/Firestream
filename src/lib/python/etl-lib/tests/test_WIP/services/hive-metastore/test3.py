from hive_metastore_client.builders import DatabaseBuilder
from hive_metastore_client import HiveMetastoreClient
import os

HIVE_HOST = os.environ.get('METASTORE_URL')
HIVE_PORT = os.environ.get('METASTORE_PORT')

DATABASE_NAME = 'new_db'
FILEPATH = 's3a://path/to/file'

try:
    database = DatabaseBuilder(name=DATABASE_NAME).build()
    with HiveMetastoreClient(HIVE_HOST, HIVE_PORT) as hive_metastore_client:
        hive_metastore_client.create_database(database) 
except Exception as e:
    print(e)


"""
    The thrift Table object requires others objects as arguments.
    Use the builders for creating each of them.
    Some arguments are optional when creating a thrift object.
    Check each Builder constructor for more information.

    Due to a bug in Hive Metastore server we need to enforce the parameter
     EXTERNAL=TRUE when creating an external table. You can either use the
     method `create_external_table` with the table object or declare the two
     table parameters before calling the method create_table.
"""

from hive_metastore_client import HiveMetastoreClient
from hive_metastore_client.builders import (
    ColumnBuilder,
    SerDeInfoBuilder,
    StorageDescriptorBuilder,
    TableBuilder,
)



# You must create a list with the columns
columns = [
    ColumnBuilder("id", "string", "col comment").build(),
    ColumnBuilder("client_name", "string").build(),
    ColumnBuilder("amount", "string").build(),
    ColumnBuilder("year", "string").build(),
    ColumnBuilder("month", "string").build(),
    ColumnBuilder("day", "string").build(),
]

# If you table has partitions create a list with the partition columns
# This list is similar to the columns list, and the year, month and day
# columns are the same.
partition_keys = [
    ColumnBuilder("year", "string").build(),
    ColumnBuilder("month", "string").build(),
    ColumnBuilder("day", "string").build(),
]

serde_info = SerDeInfoBuilder(
    serialization_lib="org.apache.hadoop.hive.ql.io.parquet.serde.ParquetHiveSerDe"
).build()

storage_descriptor = StorageDescriptorBuilder(
    columns=columns,
    location=FILEPATH,
    input_format="org.apache.hadoop.hive.ql.io.parquet.MapredParquetInputFormat",
    output_format="org.apache.hadoop.hive.ql.io.parquet.MapredParquetOutputFormat",
    serde_info=serde_info,
).build()

table = TableBuilder(
    table_name="orders",
    db_name=DATABASE_NAME,
    owner="owner name",
    storage_descriptor=storage_descriptor,
    partition_keys=partition_keys,
).build()

with HiveMetastoreClient(HIVE_HOST, HIVE_PORT) as hive_metastore_client:
    # Creating new table from thrift table object
    hive_metastore_client.create_external_table(table)

