
FROM ubuntu:20.04 as base


## Base Container ##
# Combines downloading of external resources in one place
# Allows for efficient multistage build with minimum network activity

RUN apt-get update && apt-get -y install wget
WORKDIR /tmp



ARG SPARK_VERSION=3.3.1
ARG HADOOP_VERSION=3.3.4
ARG JAVA_VERSION=11



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






# TODO configure Nvidia packages in a seperate build step

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


FROM python:3.8-slim-bullseye


ARG SPARK_VERSION=3.3.1
ARG HADOOP_VERSION=3.3.4
ARG JAVA_VERSION=11

### Java ###
# Default to UTF-8 file.encoding (for Java)
ENV LANG en_US.UTF-8
# TODO use the openJDK image instead of Microsoft
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

# Set Hadoop default logging to WARN
RUN sed -i 's/hadoop.root.logger=INFO,console/hadoop.root.logger=WARN,console/g' $HADOOP_HOME/etc/hadoop/log4j.properties

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



# [Option] Install zsh
ARG INSTALL_ZSH="true"
# [Option] Upgrade OS packages to their latest versions
ARG UPGRADE_PACKAGES="false"


# Install needed packages and setup non-root user. Use a separate RUN statement to add your
# own dependencies. A user of "automatic" attempts to reuse an user ID if one already exists.
# Press the easy button and make this the user ID that is used by default in debian for the first user (1000:1000)
ENV USERNAME=vscode
ENV USER_UID=1000
ENV USER_GID=$USER_UID

# Copy install scripts to image
COPY ./bin /tmp/library-scripts/
COPY requirements.txt /tmp/library-scripts/requirements.txt

# ### Scala Build Tools ###
RUN /bin/bash /tmp/library-scripts/sbt-debian.sh

# Install VS Code remote development container features
RUN /bin/bash /tmp/library-scripts/common-debian.sh "${INSTALL_ZSH}" "${USERNAME}" "${USER_UID}" "${USER_GID}" "${UPGRADE_PACKAGES}" "true" "true"

### Docker ###
# [Option] Enable non-root Docker access in container
ARG ENABLE_NONROOT_DOCKER="true"
# [Option] Use the OSS Moby Engine instead of the licensed Docker Engine
ARG USE_MOBY="true"
# [Option] Engine/CLI Version
ARG DOCKER_VERSION="latest"
# Enable new "BUILDKIT" mode for Docker CLI (set permanently)
ENV DOCKER_BUILDKIT=1

# ### Docker from Docker
RUN /bin/bash /tmp/library-scripts/docker-from-docker-debian.sh "${ENABLE_NONROOT_DOCKER}" "/var/run/docker-host.sock" "/var/run/docker.sock" "${USERNAME}"


### Kubectl and Helm ###
RUN /bin/bash /tmp/library-scripts/kubectl-helm-debian.sh

### KIND (Kubernetes in Docker) ###
RUN /bin/bash /tmp/library-scripts/install_kind.sh


# # ## Python ###

# ARG TARGET_PYTHON_INSTALL_PATH=/usr/local/python
# # Setup default python tools in a venv via pipx to avoid conflicts
# ENV PIPX_HOME=/usr/local/py-utils
# RUN apt-get update && /bin/bash /tmp/library-scripts/python-debian.sh "${PYTHON_VERSION}" "${TARGET_PYTHON_INSTALL_PATH}" "${PIPX_HOME}" "${USERNAME}" "true" "true" "false" "true"


# # ## Install PySpark
# #TODO why does this fail?
# # RUN cd $SPARK_HOME/python && /usr/local/python/bin/python -m pip install .

# # Install python dependencies to site-packages
RUN python -m pip install -r /tmp/library-scripts/requirements.txt

# Install additional OS packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    librdkafka-dev \
    librdkafka++1 \
    librdkafka1 \
    build-essential \
    iputils-ping \
    dnsutils \
    apt-transport-https \
    ca-certificates \
    gnupg \
    stress \
    netcat \
    postgresql-client




# Cleanup
RUN rm -rf /tmp/library-scripts/
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Use the host's storage for persisting containers built inside this container
VOLUME [ "/var/lib/docker" ]

# Persist K8s StatefulSets
VOLUME ["/var/lib/docker/k8s"]

WORKDIR /workspace
COPY entrypoint.sh /entrypoint.sh
RUN chmod + /entrypoint.sh
ENTRYPOINT ["/bin/bash", "/entrypoint.sh"]