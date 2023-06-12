
# dependencies:latest
ARG BASE_IMAGE
ARG SPARK_HOME=${SPARK_HOME}
ARG HADOOP_HOME=${HADOOP_HOME}

FROM $BASE_IMAGE as dependencies

FROM eclipse-temurin:11-jre-alpine as runtime

ARG SPARK_HOME
ARG HADOOP_HOME

WORKDIR /spark

# Hadoop: Copy previously fetched runtime components
COPY --from=dependencies ${HADOOP_HOME} ${HADOOP_HOME}
# Spark: Copy previously fetched runtime components
COPY --from=dependencies ${SPARK_HOME} ${SPARK_HOME}



EXPOSE 10000

ENTRYPOINT java \
 -Duser.timezone=Etc/UTC \
 -Xmx512m \
#  -cp "${SPARK_DIR}/conf:${SPARK_DIR}/jars/*" \
 org.apache.spark.deploy.SparkSubmit \
 --master local[1] \
 --conf spark.executor.extraJavaOptions=-Duser.timezone=Etc/UTC \
 --conf spark.eventLog.enabled=false \
 --class org.apache.spark.sql.hive.thriftserver.HiveThriftServer2 \
 --name "Thrift JDBC/ODBC Server" \
 --executor-memory 512m \
 spark-internal