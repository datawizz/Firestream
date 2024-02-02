

import os

from pyspark.sql import *
from pyspark import SparkConf
import pynessie


_CATALOG = "dev_catalog"
URI=os.environ.get("NESSIE_SERVER_URI")
URI="http://nessie.default.svc.cluster.local:19120/api/v1"
print(URI)


conf = SparkConf()
# we need iceberg libraries and the nessie sql extensions
conf.set(
    "spark.jars.packages",
    f"org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.1,org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1",
)
# ensure python <-> java interactions are w/ pyarrow
conf.set("spark.sql.execution.pyarrow.enabled", "true")
# create catalog dev_catalog as an iceberg catalog
conf.set("spark.sql.catalog.dev_catalog", "org.apache.iceberg.spark.SparkCatalog")
# tell the dev_catalog that its a Nessie catalog
conf.set("spark.sql.catalog.dev_catalog.catalog-impl", "org.apache.iceberg.nessie.NessieCatalog")
# set the location for Nessie catalog to store data. Spark writes to this directory
conf.set("spark.sql.catalog.dev_catalog.warehouse", "file://" + os.getcwd() + "/spark_warehouse/iceberg")
# set the location of the nessie server. There are many ways to run it (see https://projectnessie.org/try/)
conf.set("spark.sql.catalog.dev_catalog.uri", URI)
# default branch for Nessie catalog to work on
conf.set("spark.sql.catalog.dev_catalog.ref", "main")
# use no authorization. Options are NONE AWS BASIC and aws implies running Nessie on a lambda
conf.set("spark.sql.catalog.dev_catalog.auth_type", "NONE")
# enable the extensions for both Nessie and Iceberg
conf.set(
    "spark.sql.extensions",
    "org.apache.iceberg.spark.extensions.IcebergSparkSessionExtensions,org.projectnessie.spark.extensions.NessieSparkSessionExtensions",
)
# finally, start up the Spark server
spark = SparkSession.builder.config(conf=conf).getOrCreate()
print("Spark Running")


spark.sql(f"CREATE BRANCH IF NOT EXISTS work IN {_CATALOG} FROM main" ).show()
spark.sql(f"USE REFERENCE work IN {_CATALOG}").show()
spark.sql(
f"""CREATE TABLE IF NOT EXISTS {_CATALOG}.testing.city (
    C_CITYKEY BIGINT, C_NAME STRING, N_NATIONKEY BIGINT, C_COMMENT STRING
) USING iceberg PARTITIONED BY (N_NATIONKEY)
"""
).collect()
#spark.sql(f"INSERT INTO {_CATALOG}.testing.city VALUES (1, 'a', 1, 'comment')")
spark.sql(f"SHOW TABLES IN {_CATALOG}").show()