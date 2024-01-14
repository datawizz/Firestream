# Lake FS

Lake FS is a filesystem for Data Lakes which brings git like semantics to massive object stores. This enables transactions over multiple tables when using SQL and Spark even over huge datasets.

Lake FS provides a S3 compatible API which abstracts cloud provider APIs while curating the view of the files. In the case of S3 (or MinIO) this is a passthrough where Spark can read the files directly without additional overhead while metadata API calls are handled by Lake FS.

In this way Lake FS supports far more than just Spark, such as Duck DB in the browser! #TODO

Lake FS is agnostic to the underlying file format and happily supports Delta Lake among others.
