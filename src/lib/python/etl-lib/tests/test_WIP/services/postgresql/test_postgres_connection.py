import os
from sqlalchemy import create_engine
from sqlalchemy.exc import OperationalError

# Read environment variables
POSTGRES_USER = os.environ['POSTGRES_USER']
POSTGRES_PASSWORD = os.environ['POSTGRES_PASSWORD']
POSTGRES_DEFAULT_DB = os.environ['POSTGRES_DEFAULT_DB']
POSTGRES_URL = os.environ['POSTGRES_URL']
POSTGRES_PORT = os.environ['POSTGRES_PORT']

def test_connection():

    # Create a connection string for SQLAlchemy
    connection_string = f"postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_URL}:{POSTGRES_PORT}/{POSTGRES_DEFAULT_DB}"

    # Create a connection to the database
    engine = create_engine(connection_string)

    try:
        # Test connection
        with engine.connect() as connection:
            print("Connected!")
    except OperationalError as e:
        print(f"Error: {e}")

    # Close the connection
    engine.dispose()

if __name__ == "__main__":
    test_connection()