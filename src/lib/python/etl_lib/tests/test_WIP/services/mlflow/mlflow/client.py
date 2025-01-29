"""
https://github.com/mlflow/mlflow/blob/5fc8473ae9b252ee3c47b9da7a7e0cd8a7e59d90/mlflow/pyfunc/__init__.py#L756


Use the PyFunc UDF in spark for inference (as a MVP)

Use the "local" environment to run in the ETL Lib dockerfile (which has all dependencies)


ML Flow expects the trained model to be in S3 or a local directory.




** Actions **

1. Create a PostgreSQL container in Helm for ML Flow backend


1. Create a Dockerfile + Helm Chart to deploy ML Flow server as a K8 container
    Use the MinIO S3 implementation, setting the MLFLOW_S3_LOCAL_ENDPOINT_URL env var to http://minio.local.cluster:9000

1.1 Port Forward into the container to allow access locally
1.2 View the UI
1.3 Make logging requests to the MLFlow server process

"""
