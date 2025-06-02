import os
import uuid
import re
import pytest
from sqlalchemy import create_engine, text, MetaData, Table, Column, String
from sqlalchemy.exc import OperationalError

# Read environment variables
POSTGRES_USER = os.environ['POSTGRES_USER']
POSTGRES_PASSWORD = os.environ['POSTGRES_PASSWORD']
POSTGRES_DEFAULT_DB = os.environ['POSTGRES_DEFAULT_DB']
POSTGRES_URL = os.environ['POSTGRES_URL']
POSTGRES_PORT = os.environ['POSTGRES_PORT']


def setup():
    # Create a connection string for SQLAlchemy
    connection_string = f"postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_URL}:{POSTGRES_PORT}/{POSTGRES_DEFAULT_DB}"

    # Create a connection to the database
    engine = create_engine(connection_string)
    metadata = MetaData()
    print(metadata.__dict__)

    return engine, metadata

def convert_uuid_to_pg_schema_name(uuid_value):
    # Remove hyphens from the UUID
    uuid_no_hyphen = re.sub(r'-', '', str(uuid_value))

    # Ensure the schema name starts with a letter
    schema_name = "a" + uuid_no_hyphen

    return schema_name

# Test for database operations
def test_database_operations():

    engine, metadata = setup()
    schema_name = convert_uuid_to_pg_schema_name(str(uuid.uuid4()))
    table_name = convert_uuid_to_pg_schema_name(str(uuid.uuid4()))
    row_id = convert_uuid_to_pg_schema_name(str(uuid.uuid4()))
    new_id = convert_uuid_to_pg_schema_name(str(uuid.uuid4()))

    def execute_query(query, **params):

        with engine.connect() as connection:
            result = connection.execute(text(query), **params)
            
        return result

    def create_schema(schema_name):
        execute_query(f"CREATE SCHEMA {schema_name}")

    def create_table(schema_name, table_name):
        table = Table(table_name, metadata,
                    Column('id', String, primary_key=True),
                    schema=schema_name)
        table.create(engine)

    def insert_row(schema_name, table_name, row_id):
        execute_query(f"INSERT INTO {schema_name}.{table_name} (id) VALUES ('{row_id}')")

    def update_row(schema_name, table_name, old_id, new_id):
        execute_query(f"UPDATE {schema_name}.{table_name} SET id = '{new_id}' WHERE id = '{old_id}'")

    def delete_row(schema_name, table_name, row_id):
        execute_query(f"DELETE FROM {schema_name}.{table_name} WHERE id = '{row_id}'")

    def drop_table(schema_name, table_name):
        execute_query(f"DROP TABLE {schema_name}.{table_name}")

    def drop_schema(schema_name):
        execute_query(f"DROP SCHEMA {schema_name} CASCADE")


    create_schema(schema_name)
    create_table(schema_name, table_name)
    insert_row(schema_name, table_name, row_id)
    update_row(schema_name, table_name, row_id, new_id)
    delete_row(schema_name, table_name, new_id)
    drop_table(schema_name, table_name)
    drop_schema(schema_name)

    
    engine.dispose()




if __name__ == "__main__":
    test_database_operations()
