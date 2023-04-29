

# Lake FS

Lake File System is git for data. Each series of modifications to the dataset is committed to a metadata store in PostgreSQL.

Compatibility is maintained with Hadoop using the S3 Jars. Metadata writing is optimized as to pass through the write to the S3 storage.

