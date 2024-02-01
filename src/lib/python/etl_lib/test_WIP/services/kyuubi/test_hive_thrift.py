



# hive://hive@{hostname}:{port}/{database}


_SERVER = "kyuubi-thrift-binary.default.svc.cluster.local"
_PORT = 10009

from pyhive import hive
from TCLIService.ttypes import TOperationState
cursor = hive.connect(host=_SERVER, port=_PORT).cursor()
cursor.execute('SHOW DATABASES', _async=True)

status = cursor.poll().operationState
while status in (TOperationState.INITIALIZED_STATE, TOperationState.RUNNING_STATE):
    logs = cursor.fetch_logs()
    for message in logs:
        print(message)

    # If needed, an asynchronous query can be cancelled at any time with:
    # cursor.cancel()

    status = cursor.poll().operationState

print(cursor.fetchall())

# from pyhive import hive
# from TCLIService.ttypes import TOperationState

# def connect_to_hive(host=_SERVER, port=_PORT):
#     """
#     Function to connect to Hive server
#     """
#     try:
#         # Establish the connection
#         conn = hive.connect(host=host, port=port)
#         cursor = conn.cursor()
#         print("Connection Successful!")
#         return cursor
#     except Exception as e:
#         print(f"Connection Failed! Error: {e}")
#         return None

# def list_databases(cursor):
#     """
#     Function to list all the databases
#     """
#     if cursor is not None:
#         try:
#             cursor.execute('SHOW DATABASES')
#             print("Databases:")
#             for database in cursor.fetchall():
#                 print(database)
#         except Exception as e:
#             print(f"Failed to fetch databases! Error: {e}")

# def list_tables(cursor, database='default'):
#     """
#     Function to list all the tables in a specific database
#     """
#     if cursor is not None:
#         try:
#             cursor.execute(f'USE {database}')
#             cursor.execute('SHOW TABLES')
#             print(f"Tables in {database}:")
#             for table in cursor.fetchall():
#                 print(table)
#         except Exception as e:
#             print(f"Failed to fetch tables from {database}! Error: {e}")

# def main():
#     # connect to hive
#     cursor = connect_to_hive()
    
#     # list databases
#     list_databases(cursor)
    
#     # list tables
#     list_tables(cursor)

# if __name__ == "__main__":
#     main()
