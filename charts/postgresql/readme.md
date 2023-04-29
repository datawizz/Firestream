

# PostgreSQL

PostgreSQL is used for persistance by:

0. For Role Based Access Control and data governance control (implemented in Auth_Lib)
1. LakeFS for storing git history of data
2. For Airflow storing active job state (but not logs! Those are in S3)
3. For PGVector for serving indexes over the data in S3