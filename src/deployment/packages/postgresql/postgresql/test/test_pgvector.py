import os
from sqlalchemy import create_engine, Column, Integer, String
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy.sql import text
from pgvector.sqlalchemy import Vector

# Set up the database connection
POSTGRES_USER = os.environ['POSTGRES_USER']
POSTGRES_PASSWORD = os.environ['POSTGRES_PASSWORD']
POSTGRES_DEFAULT_DB = os.environ['POSTGRES_DEFAULT_DB']
POSTGRES_URL = os.environ['POSTGRES_URL']
POSTGRES_PORT = os.environ['POSTGRES_PORT']

# Create the database engine
engine = create_engine(f"postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_URL}:{POSTGRES_PORT}/{POSTGRES_DEFAULT_DB}")

# Create the database model
Base = declarative_base()

class DataItem(Base):
    __tablename__ = 'data_items'
    id = Column(Integer, primary_key=True)
    name = Column(String)
    vector = Column(Vector(3))

# Create the table
Base.metadata.create_all(engine)

# Create a session
Session = sessionmaker(bind=engine)
session = Session()

# Insert data (Create)
new_item = DataItem(name='Item1', vector=[1, 2, 3])
session.add(new_item)
session.commit()

# Read data
item = session.query(DataItem).filter_by(name='Item1').first()
print(f"Read data: {item.name} with vector {item.vector}")

# Update data
item.vector = [-1, -2, -3]
session.commit()
print(f"Updated data: {item.name} with vector {item.vector}")

# Delete data
session.delete(item)
session.commit()

# Check if the item was deleted
deleted_item = session.query(DataItem).filter_by(name='Item1').first()
if deleted_item is None:
    print("Item1 deleted successfully")

session.close()
