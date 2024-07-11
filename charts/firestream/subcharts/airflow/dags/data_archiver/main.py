# Data Archiver Service

# This service is responsible for archiving data from Kafka topics to S3.
# It makes use of the Spark Structured Streaming API to read from Kafka topics and write to S3.
# Data is formatted as Delta tables backed by Lake FS.
# Borrows heavily from ETL Lib.

