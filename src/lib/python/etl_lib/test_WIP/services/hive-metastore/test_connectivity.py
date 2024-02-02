import socket
import sys

def check_connectivity(host, port):
    try:
        sock = socket.create_connection((host, port), timeout=5)
        print(f"Successfully connected to Hive metastore at {host}:{port}")
        sock.close()
    except (socket.error, socket.timeout) as e:
        print(f"Error connecting to Hive metastore at {host}:{port}: {e}")
        sys.exit(1)

if __name__ == "__main__":
    hive_metastore_host = "hive-metastore.default.svc.cluster.local"
    hive_metastore_port = 9083

    check_connectivity(hive_metastore_host, hive_metastore_port)
