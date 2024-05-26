# Data Archiver API

## A filestore

When run in API mode Data Archiver writes to a filestore upon receiving a post of a file.

This uses a persistant SQL interface to write the data of the file as binary and using spark to write to the filestore.