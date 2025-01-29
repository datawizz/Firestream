
"""
# Goal, create a set of functions for interacting with the super set API
# It should be able to:
# 1. Create a new database (SparkSQL). #TODO how to handle security?
# 2. Create a new table (SparkSQL).
# 3. Create a new slice
# 4. Create a new chart
# 5. Create a new dashboard

Each of the should be created from DataModels that are consumed one at a time to build the requested resources.


Load examples using the kubectl command below:
kubectl exec -it superset-69459c794f-fqxm6 -- superset load_examples






"""