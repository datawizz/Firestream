#https://blog.devgenius.io/hive-metastore-a9dc9e139cf2
#TODO there is no license associated with this? Might need a license file


FROM openjdk:16-slim

ARG HADOOP_VERSION=3.3.1
ARG HIVE_METASTORE_VERSION=3.0.0
ARG POSTGRES_CONNECTOR_VERSION=42.3.1

#TODO pull this from environment
ENV HADOOP_HOME="/opt/hadoop" 
ENV METASTORE_HOME="/opt/hive-metastore"
ENV PATH="/opt/spark/bin:/opt/hadoop/bin:${METASTORE_HOME}/lib/:${PATH}"

WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN \
    echo "Install dependencies" && \
    build_deps="curl" && \
    apt-get update -y && \
    apt-get install -y $build_deps --no-install-recommends && \
    echo "Adding Hadoop Jars " && \
    curl https://archive.apache.org/dist/hadoop/core/hadoop-$HADOOP_VERSION/hadoop-$HADOOP_VERSION.tar.gz \
    | tar xvz -C /opt/ && \
    ln -s /opt/hadoop-$HADOOP_VERSION /opt/hadoop && \
    rm -r /opt/hadoop/share/doc && \
    echo "Adding S3 Jars" && \
    ln -s /opt/hadoop/share/hadoop/tools/lib/hadoop-aws* /opt/hadoop/share/hadoop/common/lib/ && \
    ln -s /opt/hadoop/share/hadoop/tools/lib/aws-java-sdk* /opt/hadoop/share/hadoop/common/lib/ && \
    echo "Install Standalone metastore binary" && \
    curl https://downloads.apache.org/hive/hive-standalone-metastore-$HIVE_METASTORE_VERSION/hive-standalone-metastore-$HIVE_METASTORE_VERSION-bin.tar.gz \
    | tar xvz -C /opt/ && \
    ln -s /opt/apache-hive-metastore-$HIVE_METASTORE_VERSION-bin ${METASTORE_HOME} && \
    echo "Fix 'java.lang.NoSuchMethodError: com.google.common.base.Preconditions.checkArgument'" && \
    echo "Keep this until this lands: https://issues.apache.org/jira/browse/HIVE-22915" && \
    rm /opt/apache-hive-metastore-$HIVE_METASTORE_VERSION-bin/lib/guava-19.0.jar && \
    cp /opt/hadoop-$HADOOP_VERSION/share/hadoop/hdfs/lib/guava-27.0-jre.jar /opt/apache-hive-metastore-$HIVE_METASTORE_VERSION-bin/lib/ && \
    echo "Install the database connector" && \
    curl -L https://jdbc.postgresql.org/download/postgresql-$POSTGRES_CONNECTOR_VERSION.jar --output /opt/postgresql-$POSTGRES_CONNECTOR_VERSION.jar && \
    ln -s /opt/postgresql-$POSTGRES_CONNECTOR_VERSION.jar /opt/hadoop/share/hadoop/common/lib/ && \
    ln -s /opt/postgresql-$POSTGRES_CONNECTOR_VERSION.jar /opt/hive-metastore/lib/ && \
    echo "Purge build artifacts" && \
    apt-get purge -y --auto-remove $build_deps && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY entrypoint.sh ${METASTORE_HOME}/bin/
COPY metastore-log4j2.properties  ${METASTORE_HOME}/conf
RUN chmod 775 ${METASTORE_HOME}/bin/entrypoint.sh

EXPOSE 9083

WORKDIR ${METASTORE_HOME}

CMD ["bin/entrypoint.sh"]
