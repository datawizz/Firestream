

# Download https://drive.google.com/uc?export=download&id=1TqleXec_OykOYFREKKtschzY29dUcVAQ

# Unzip 
# unzip spider.zip

from pyspark.sql import SparkSession

spark = SparkSession.builder\
           .config('spark.jars.packages', 'org.xerial:sqlite-jdbc:3.34.0')\
           .getOrCreate()

df = spark.read.format('jdbc') \
        .options(driver='org.sqlite.JDBC', dbtable='election',
                 url='jdbc:sqlite:/workspace/plugins/data_plugins/spider/database/election/election.sqlite')\
        .load()

df.show()