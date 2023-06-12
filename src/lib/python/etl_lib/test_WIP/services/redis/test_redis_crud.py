







# bootstrap_server = "redis-cluster-1680920766-headless.default.svc.cluster.local"


# import redis

# class RedisCRUD:
#     def __init__(self, host=bootstrap_server, port=6379):
#         self.client = redis.Redis(host=host, port=port, decode_responses=True)

#     def create(self, key, value):
#         if self.client.exists(key):
#             raise ValueError(f"Key '{key}' already exists.")
#         self.client.set(key, value)

#     def read(self, key):
#         if not self.client.exists(key):
#             raise KeyError(f"Key '{key}' not found.")
#         return self.client.get(key)

#     def update(self, key, value):
#         if not self.client.exists(key):
#             raise KeyError(f"Key '{key}' not found.")
#         self.client.set(key, value)

#     def delete(self, key):
#         if not self.client.exists(key):
#             raise KeyError(f"Key '{key}' not found.")
#         self.client.delete(key)

# # Example usage
# if __name__ == "__main__":
#     redis_crud = RedisCRUD()

#     # Create a new key-value pair
#     redis_crud.create("test_key", "test_value")

#     # Read the value of the key
#     print(redis_crud.read("test_key"))

#     # Update the value of the key
#     redis_crud.update("test_key", "updated_value")
#     print(redis_crud.read("test_key"))

#     # Delete the key-value pair
#     redis_crud.delete("test_key")


import os
import redis
from redis.cluster import RedisCluster

# Variables
REDIS_PASSWORD = os.getenv("REDIS_PASSWORD")
REDIS_HOST = "redis-cluster.default.svc.cluster.local"
REDIS_PORT = 6379

from redis.cluster import RedisCluster

rc = RedisCluster(host=REDIS_HOST, port=REDIS_PORT, password=REDIS_PASSWORD)

print(rc.get_nodes())
# [[host=127.0.0.1,port=16379,name=127.0.0.1:16379,server_type=primary,redis_connection=Redis<ConnectionPool<Connection<host=127.0.0.1,port=16379,db=0>>>], ...

r = rc.set('foo', 'bar')
print(r)
# True

r = rc.get('foo')
print(r)
# b'bar'





## Redis OM

import datetime
from typing import Optional

from pydantic import EmailStr

from redis_om import HashModel


class Customer(HashModel):
    first_name: str
    last_name: str
    email: EmailStr
    join_date: datetime.date
    age: int
    bio: Optional[str]

import datetime
from typing import Optional

from pydantic import EmailStr

from redis_om import HashModel


class Customer(HashModel):
    first_name: str
    last_name: str
    email: EmailStr
    join_date: datetime.date
    age: int
    bio: Optional[str]


# First, we create a new `Customer` object:
andrew = Customer(
    first_name="Andrew",
    last_name="Brookins",
    email="andrew.brookins@example.com",
    join_date=datetime.date.today(),
    age=38,
    bio="Python developer, works at Redis, Inc."
)

# The model generates a globally unique primary key automatically
# without needing to talk to Redis.
print(andrew.pk)
# > "01FJM6PH661HCNNRC884H6K30C"

# We can save the model to Redis by calling `save()`:
andrew.save()

# Expire the model after 2 mins (120 seconds)
andrew.expire(120)

# To retrieve this customer with its primary key, we use `Customer.get()`:
assert Customer.get(andrew.pk) == andrew






# def main():
#     # Connect to the Redis cluster
#     redis_cluster = redis.StrictRedisCluster(
#         host=REDIS_HOST,
#         password=REDIS_PASSWORD,
#         decode_responses=True,
#         skip_full_coverage_check=True
#     )

#     # Set a key-value pair in the Redis cluster
#     redis_cluster.set("my_key", "my_value")

#     # Retrieve the value for the key
#     value = redis_cluster.get("my_key")
#     print(f"Value for 'my_key': {value}")

# if __name__ == "__main__":
#     main()


