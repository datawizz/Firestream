


# 1. Pull version numbers from the environment variables
# 2. Download the files to the local filesystem




wget https://dlcdn.apache.org/solr/solr/9.2.0/solr-9.2.0-src.tgz

wget https://dlcdn.apache.org/solr/solr-operator/v0.6.0/solr-operator-v0.6.0.tgz



ARG SPARK_VERSION=3.3.2
ARG SPARK_SHA512="347fd9029128b12e7b05e9cd7948a5b571a57f16bbbbffc8ad4023b4edc0e127cffd27d66fcdbf5f926fa33362a2ae4fc0a8d59ab3abdaa1d4c4ef1e23126932  spark-3.3.2-bin-without-hadoop.tgz"
ARG HADOOP_VERSION=3.3.4
ARG HADOOP_SHA512="ca5e12625679ca95b8fd7bb7babc2a8dcb2605979b901df9ad137178718821097b67555115fafc6dbf6bb32b61864ccb6786dbc555e589694a22bf69147780b4  hadoop-3.3.4.tar.gz"


## Spark ##
# The "without hadoop" binary is used so that *any* hadoop version can be supplied and linked to Spark
RUN wget https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz
RUN echo $SPARK_SHA512 | sha512sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
    && tar xvzf spark-${SPARK_VERSION}-bin-without-hadoop.tgz

## Hadoop ##
RUN wget https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz
RUN echo $HADOOP_SHA512 | sha512sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
    && tar xvzf hadoop-${HADOOP_VERSION}.tar.gz
