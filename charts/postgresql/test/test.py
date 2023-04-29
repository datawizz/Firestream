




import os
from sqlalchemy import create_engine, Column, Integer, String, update, select
from sqlalchemy.orm import declarative_base, sessionmaker

# Set up the database connection
POSTGRES_USER = os.environ['POSTGRES_USER']
POSTGRES_PASSWORD = os.environ['POSTGRES_PASSWORD']
POSTGRES_DEFAULT_DB = os.environ['POSTGRES_DEFAULT_DB']
POSTGRES_URL = os.environ['POSTGRES_URL']
POSTGRES_PORT = os.environ['POSTGRES_PORT']




DATABASE_URI = f"postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_URL}:{POSTGRES_PORT}/{POSTGRES_DEFAULT_DB}"
engine = create_engine(DATABASE_URI)
Session = sessionmaker(bind=engine)

# Define the database schema using SQLAlchemy ORM
Base = declarative_base()

class User(Base):
    __tablename__ = 'users'

    id = Column(Integer, primary_key=True)
    name = Column(String)
    age = Column(Integer)

Base.metadata.create_all(engine)

# CRUD functions
def create_user(name, age):
    with Session() as session:
        user = User(name=name, age=age)
        session.add(user)
        session.commit()

def read_users():
    with Session() as session:
        users = session.execute(select(User)).scalars().all()
    return users

def update_user(user_id, new_name, new_age):
    with Session() as session:
        user = session.get(User, user_id)
        if user:
            user.name = new_name
            user.age = new_age
            session.commit()

def delete_user(user_id):
    with Session() as session:
        user = session.get(User, user_id)
        if user:
            session.delete(user)
            session.commit()

# Use the CRUD functions to interact with the database
create_user("John", 25)
users = read_users()
print("Users before update:", users)

update_user(1, "John Doe", 26)
users = read_users()
print("Users after update:", users)

delete_user(1)
users = read_users()
print("Users after delete:", users)