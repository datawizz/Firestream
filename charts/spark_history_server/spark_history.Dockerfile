FROM ubuntu:20.04 as base
## Base Container ##
# Combines downloading of external resources in one place
# Allows for efficient multistage build with minimum network activity

RUN apt-get update && apt-get -y install wget
WORKDIR /tmp



ARG SPARK_VERSION=3.3.1
ARG HADOOP_VERSION=3.3.4
ARG PYTHON_VERSION="3.9.14"
ARG JAVA_VERSION=11


### Java ###
# Default to UTF-8 file.encoding (for Java)
ENV LANG en_US.UTF-8
ENV JAVA_HOME /usr/lib/jvm/msopenjdk-${JAVA_VERSION}-amd64
ENV PATH "${JAVA_HOME}/bin:${PATH}"
COPY --from=mcr.microsoft.com/openjdk/jdk:11-ubuntu $JAVA_HOME $JAVA_HOME

## Spark
# The "without hadoop" binary is used so that any* hadoop version can be supplied and linked to Spark
RUN wget https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz \
    && tar xvzf spark-${SPARK_VERSION}-bin-without-hadoop.tgz

# TODO hash the binary to confirm that it is the expected one
RUN wget https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz.sha512



## Hadoop
RUN wget https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz \
    && tar xvzf hadoop-${HADOOP_VERSION}.tar.gz

# TODO hash the binary to confirm that it is the expected one
RUN wget https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz.sha512


# Python
# TODO PIP Download for python dependencies to avoid recaching them
# https://pip.pypa.io/en/stable/cli/pip_download/



# Spark History Server

FROM base as spark



ENV DEBIAN_FRONTEND=noninteractive

# Set commit hash
# RUN git rev-parse HEAD > commit_hash

# # Install Spark Dependencies and Prepare Spark Runtime Environment
# RUN set -ex && \
#     apt-get update && \
#     ln -s /lib /lib64 && \
#     apt install -y bash tini libc6 libpam-modules libnss3 wget python3 python3-pip && \
#     mkdir -p /opt/hadoop && \
#     mkdir -p /opt/spark && \
#     mkdir -p /opt/spark/examples && \
#     mkdir -p /opt/spark/work-dir && \
#     touch /opt/spark/RELEASE && \
#     rm /bin/sh && \
#     ln -sv /bin/bash /bin/sh && \
#     ln -sv /usr/bin/tini /sbin/tini && \
#     echo "auth required pam_wheel.so use_uid" >> /etc/pam.d/su && \
#     chgrp root /etc/passwd && chmod ug+rw /etc/passwd && \
#     ln -sv /usr/bin/python3 /usr/bin/python && \
#     ln -sv /usr/bin/pip3 /usr/bin/pip \
#     rm -rf /var/cache/apt/*


# Adopt arguments defined above
# TODO how do variables transfer between multistage builds?
ARG SPARK_VERSION=3.3.1
ARG HADOOP_VERSION=3.3.4
ARG PYTHON_VERSION="3.9.14"
ARG JAVA_VERSION=11

#TODO DRY this up
### Java ###
# Default to UTF-8 file.encoding (for Java)
ENV LANG en_US.UTF-8
ENV JAVA_HOME /usr/lib/jvm/msopenjdk-${JAVA_VERSION}-amd64
ENV PATH "${JAVA_HOME}/bin:${PATH}"
COPY --from=mcr.microsoft.com/openjdk/jdk:11-ubuntu $JAVA_HOME $JAVA_HOME



# Hadoop: Copy previously fetched runtime components
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/bin /opt/hadoop/bin
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/etc /opt/hadoop/etc
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/include /opt/hadoop/include
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/lib /opt/hadoop/lib
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/libexec /opt/hadoop/libexec
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/sbin /opt/hadoop/sbin
COPY --from=base /tmp/hadoop-${HADOOP_VERSION}/share /opt/hadoop/share

# Spark: Copy previously fetched runtime components
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/bin /opt/spark/bin
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/conf /opt/spark/conf
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/data /opt/spark/data
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/examples /opt/spark/examples
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/kubernetes /opt/spark/kubernetes
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/jars /opt/spark/jars
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/python /opt/spark/python
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/R /opt/spark/R
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/sbin /opt/spark/sbin
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/yarn /opt/spark/yarn

# Spark: Copy Docker entry script
# COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/kubernetes/dockerfiles/spark/entrypoint.sh /opt/

# Spark: Copy examples, data, and tests
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/examples /opt/spark/examples
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/data /opt/spark/data
COPY --from=base /tmp/spark-${SPARK_VERSION}-bin-without-hadoop/kubernetes/tests /opt/spark/tests


# Set Hadoop environment
ENV HADOOP_HOME /opt/hadoop
ENV LD_LIBRARY_PATH $HADOOP_HOME/lib/native

# Set Spark environment
ENV SPARK_HOME /opt/spark
ENV PATH $PATH:$SPARK_HOME/bin:$HADOOP_HOME/bin
ENV SPARK_DIST_CLASSPATH /opt/hadoop/etc/hadoop:/opt/hadoop/share/hadoop/common/lib/*:/opt/hadoop/share/hadoop/common/*:/opt/hadoop/share/hadoop/hdfs:/opt/hadoop/share/hadoop/hdfs/lib/*:/opt/hadoop/share/hadoop/hdfs/*:/opt/hadoop/share/hadoop/mapreduce/lib/*:/opt/hadoop/share/hadoop/mapreduce/*:/opt/hadoop/share/hadoop/yarn:/opt/hadoop/share/hadoop/yarn/lib/*:/opt/hadoop/share/hadoop/yarn/*
ENV SPARK_CLASSPATH /opt/spark/jars/*:$SPARK_DIST_CLASSPATH

# # ### Apache Spark ###
# # Install the binaries for Spark so that "Spark Submit" works locally
# ENV SPARK_HOME=/usr/local/spark
# ENV PATH $PATH:$SPARK_HOME/bin
# RUN /bin/bash /tmp/library-scripts/spark-debian.sh "${SPARK_HOME}"



FROM spark as spark_history




ENV S3logPath='s3a//logs/spark'

# How to set environment variables to enable access to s3 bucket from within k8 cluster?


# "spark.hadoop.fs.s3a.endpoint" : TheServiceInK8
spark.history.ui.port 18080
spark.history.fs.logDirectory s3a://logs/spark
spark.history.retainedApplications

EXPOSE 18080


spark-defaults.conf





ENTRYPOINT [ "$SPARK_HOME/sbin/start-history-server.sh" ]