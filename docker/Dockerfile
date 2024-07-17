### Build Arguments ###

ARG DEPLOYMENT_MODE
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS

ARG HOST_USERNAME
ARG DOCKER_BUILDKIT=1


ARG SPARK_VERSION=3.4.3
ARG SPARK_SHA512="4959a7b8710a3c37d94f1a5bf89fcbd8c757e68059fbfab8d068fd38105e62a2217ebeb4c68a6d4d4835b8ad2a65f1d00d0d1272e083934531adee69da27651b  spark-3.4.3-bin-without-hadoop.tgz"
ARG SPARK_VERSION=3.4.3
ARG SPARK_SHA512="4959a7b8710a3c37d94f1a5bf89fcbd8c757e68059fbfab8d068fd38105e62a2217ebeb4c68a6d4d4835b8ad2a65f1d00d0d1272e083934531adee69da27651b  spark-3.4.3-bin-without-hadoop.tgz"
ARG SPARK_HOME="/opt/spark"
ARG HADOOP_VERSION=3.3.6
ARG HADOOP_SHA512="de3eaca2e0517e4b569a88b63c89fae19cb8ac6c01ff990f1ff8f0cc0f3128c8e8a23db01577ca562a0e0bb1b4a3889f8c74384e609cd55e537aada3dcaa9f8a  hadoop-3.3.6.tar.gz"
ARG HADOOP_HOME="/opt/hadoop"
ARG AIRFLOW_VERSION=2.8.1
ARG AIRFLOW_SHA512="381a88ca2ef48ec488aea1b64b91ed132bbc7032b9fede467660079daa23265fe992a99e414664faf0ad5373e0b128f3da904d03275dbf26628459f29df55ae1  apache-airflow-2.8.1-source.tar.gz"
ARG AIRFLOW_HOME="/opt/airflow"

ARG JAVA_VERSION=11
ARG PYTHON_VERSION=3.11
##TODO the python version is being installed from debian, instead of manually which puts the bells and whistles in
ARG NODE_VERSION=19
ARG NODE_SHA=""

ARG LAKEFS_VERSION=0.108.0

## Dependency Container ##

# Combines downloading of external resources in one place.
# Allows for efficient multistage build with minimum network activity
# and assertation of remote resources by good 'ol hard coded hash matching
# or version pinning where possible.

FROM debian:bookworm as dependencies

ENV DEBIAN_FRONTEND=noninteractive
ARG DOCKER_BUILDKIT

ARG SPARK_VERSION
ARG SPARK_SHA512
ARG SPARK_HOME
ARG HADOOP_VERSION
ARG HADOOP_SHA512
ARG HADOOP_HOME
ARG AIRFLOW_VERSION
ARG AIRFLOW_SHA512
ARG AIRFLOW_HOME
ARG JAVA_VERSION
ARG PYTHON_VERSION
ARG NODE_VERSION
ARG LAKEFS_VERSION
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS

# Install the packages but delete the package lists to enable caching of the layer
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    wget \
    unzip \
    zip \
    curl \
    gnupg2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

#TODO change to build
WORKDIR /tmp

## Spark ##
# The "without hadoop" binary is used so that *any* hadoop version can be supplied and linked to Spark
RUN wget https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz
RUN echo $SPARK_SHA512 | sha512sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
    && mkdir -p ${SPARK_HOME} \
    && tar xvzf spark-${SPARK_VERSION}-bin-without-hadoop.tgz -C /tmp

RUN mkdir -p ${SPARK_HOME} && mv spark-${SPARK_VERSION}-bin-without-hadoop/* ${SPARK_HOME}

## Hadoop ##
RUN wget https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz
RUN echo $HADOOP_SHA512 | sha512sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
    && tar xvzf hadoop-${HADOOP_VERSION}.tar.gz -C /tmp

RUN mkdir -p ${HADOOP_HOME} && mv /tmp/hadoop-${HADOOP_VERSION}/* ${HADOOP_HOME}

# Set Hadoop default logging to WARN (instead of INFO which is very verbose)
RUN sed -i 's/hadoop.root.logger=INFO,console/hadoop.root.logger=WARN,console/g' ${HADOOP_HOME}/etc/hadoop/log4j.properties

# Airflow
RUN wget https://dlcdn.apache.org/airflow/2.8.1/apache-airflow-2.8.1-source.tar.gz
RUN echo $AIRFLOW_SHA512 | sha512sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
    && mkdir -p ${AIRFLOW_HOME} && tar xvzf apache-airflow-2.8.1-source.tar.gz -C ${AIRFLOW_HOME}


# ## Hive Standalone Metastore ##
# RUN wget https://dlcdn.apache.org/hive/hive-standalone-metastore-${HIVE_VERSION}/hive-standalone-metastore-${HIVE_VERSION}-bin.tar.gz
# RUN echo $HIVE_SHA256 | sha256sum -c - && echo "Hash matched" || (echo "Hash didn't match" && exit 1) \
#     && tar xvzf hive-standalone-metastore-${HIVE_VERSION}-bin.tar.gz

# ### Iceberg ###
# RUN wget https://search.maven.org/remotecontent?filepath=org/apache/iceberg/iceberg-spark-runtime-3.3_2.12/1.2.1/iceberg-spark-runtime-3.3_2.12-1.2.1.jar

## Python ##

# Install Python
COPY bin/install_scripts/python-debian.sh /tmp/workspace/bin/install_scripts/python-debian.sh
RUN /bin/bash /tmp/workspace/bin/install_scripts/python-debian.sh "${PYTHON_VERSION}" "/usr/local/python" "/usr/local/py-utils" "${HOST_USERNAME}" "true" "false" "false" "false"

# Make "python" and "pip" command available
RUN ln -s /usr/local/python/bin/python /usr/bin/python
RUN ln -s /usr/local/python/bin/pip /usr/bin/pip

### Download Python packages required by the project
RUN python -m pip install --upgrade pip setuptools wheel
COPY requirements.txt /tmp/workspace/requirements.txt
RUN python -m pip download --no-cache-dir --no-deps -r /tmp/workspace/requirements.txt -d /tmp/python_packages


## NodeJS ##
# Download NPM Packages
# TODO

#TODO The javascript deps should be downloaded in the builder stage.
# Install Node.js dependencies
# RUN cd /workspace/services/javascript/dashboard && npm install
# RUN cd /workspace/services/javascript/websocket_middleware && npm install


## Nvidia ##
# Configure Nvidia Container Toolkit
# TODO

## Git ##
# Set commit hash to environment variable
# RUN git rev-parse HEAD > commit_hash

## Java ##

#TODO download java SDK and JRE using SDKman
#TODO download sbt and maven using SDKman
#TODO get the JAR files downloaded that will be used later from maven et al
# RUN bash /tmp/workspace/bin/install_scripts/sdkman.sh
# TODO should the Java versions be downloaded here?

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


### Source Code ###
# Download the source code for the project's dependencies
# This is used in downstream stages to build the project's artifacts

COPY ./bin /tmp/workspace/bin






### Devcontainer ###

# Contains everything (and the kitchen sink) required to build project artifacts and run tests

FROM debian:bookworm as devcontainer

ENV DEBIAN_FRONTEND=noninteractive

## Build Args ##
ARG DEPLOYMENT_MODE
ARG SPARK_VERSION
ARG SPARK_HOME
ARG HADOOP_VERSION
ARG HADOOP_HOME
ARG AIRFLOW_HOME
ARG AIRFLOW_SHA512
ARG AIRFLOW_VERSION
ARG JAVA_VERSION
ARG PYTHON_VERSION
ARG NODE_VERSION
ARG LAKEFS_VERSION
ARG HOST_USERNAME
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS

# Install additional OS packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # required for Python and Node interfaces to Kafka
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
    # netcat \
    postgresql-client \
    libglib2.0-0 \
    libnss3 \
    libx11-6 \
    # Headless chrome
    chromium \
    # Dev Tools
    wget \
    unzip \
    zip \
    curl \
    gnupg2 \
    ca-certificates \
    # Used to enable GPU support in the container
    pciutils \
    # pyhive Thrift API dependencies
    libsasl2-dev \
    libsasl2-modules-gssapi-mit \
    # Required for TLS Websocket support in Rust compiler
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    #Airflow dependencies
    apt-transport-https \
    apt-utils ca-certificates \
    curl \
    dumb-init \
    freetds-bin \
    gosu \
    krb5-user \
    libgeos-dev \
    ldap-utils \
    libsasl2-2 \
    libsasl2-modules \
    libxmlsec1 \
    locales \
    libffi8 \
    libldap-2.5-0 \
    libssl3 \
    netcat-openbsd \
    lsb-release \
    openssh-client \
    python3-selinux \
    rsync \
    sasl2-bin \
    sqlite3 \
    sudo \
    unixodbc \
    && rm -rf /var/lib/apt/lists/*


# Add the user
RUN useradd -m -u ${HOST_USER_ID} -g ${HOST_GROUP_ID} -o ${HOST_USERNAME} || true;



# Python
COPY --from=dependencies /tmp/workspace/bin/install_scripts/python-debian.sh /tmp/workspace/bin/install_scripts/python-debian.sh
# Install Python
RUN /bin/bash /tmp/workspace/bin/install_scripts/python-debian.sh "${PYTHON_VERSION}" "/usr/local/python" "/usr/local/py-utils" "${HOST_USERNAME}" "true" "false" "false" "false"
# Make "python" command available at the terminal
RUN ln -s /usr/local/python/bin/python /usr/bin/python
RUN ln -s /usr/local/python/bin/pip /usr/bin/pip



# Hadoop: Copy previously fetched runtime components
COPY --from=dependencies ${HADOOP_HOME} ${HADOOP_HOME}

# Spark: Copy previously fetched runtime components
COPY --from=dependencies ${SPARK_HOME} ${SPARK_HOME}

# Airflow: Copy previously fetched runtime components
COPY --from=dependencies ${AIRFLOW_HOME} ${AIRFLOW_HOME}
RUN chown -R ${HOST_USER_ID}:${HOST_GROUP_ID} ${AIRFLOW_HOME}

# Copy install scripts
COPY --from=dependencies /tmp/workspace/bin /tmp/workspace/bin


# Set Hadoop environment
ENV LD_LIBRARY_PATH $HADOOP_HOME/lib/native


# Set Spark environment
ENV SPARK_HOME /opt/spark
ENV PATH $PATH:$SPARK_HOME/bin:$HADOOP_HOME/bin
ENV SPARK_DIST_CLASSPATH /opt/hadoop/etc/hadoop:/opt/hadoop/share/hadoop/common/lib/*:/opt/hadoop/share/hadoop/common/*:/opt/hadoop/share/hadoop/hdfs:/opt/hadoop/share/hadoop/hdfs/lib/*:/opt/hadoop/share/hadoop/hdfs/*:/opt/hadoop/share/hadoop/mapreduce/lib/*:/opt/hadoop/share/hadoop/mapreduce/*:/opt/hadoop/share/hadoop/yarn:/opt/hadoop/share/hadoop/yarn/lib/*:/opt/hadoop/share/hadoop/yarn/*
ENV SPARK_CLASSPATH /opt/spark/jars/*:$SPARK_DIST_CLASSPATH




# # ### Apache Spark ###
# Install the binaries for Spark so that "Spark Submit" works locally
ENV SPARK_HOME=/usr/local/spark
ENV PATH $PATH:$SPARK_HOME/bin
# RUN /bin/bash /tmp/library-scripts/spark-debian.sh "${SPARK_HOME}"



# [Option] Install zsh
ARG INSTALL_ZSH="true"
# [Option] Upgrade OS packages to their latest versions
ARG UPGRADE_PACKAGES="false"

# Install VS Code remote development container features
RUN /bin/bash /tmp/workspace/bin/install_scripts/common-debian.sh "${INSTALL_ZSH}" "${HOST_USERNAME}" "${HOST_USER_ID}" "${HOST_GROUP_ID}" "${UPGRADE_PACKAGES}" "true" "true"

### Docker ###
# [Option] Enable non-root Docker access in container
ARG ENABLE_NONROOT_DOCKER="true"
# [Option] Use the OSS Moby Engine instead of the licensed Docker Engine
ARG USE_MOBY="true"
# [Option] Engine/CLI Version
ARG DOCKER_VERSION="latest"
# Enable new "BUILDKIT" mode for Docker CLI (set permanently)
ENV DOCKER_BUILDKIT=1

### Docker from Docker
RUN /bin/bash /tmp/workspace/bin/install_scripts/docker-from-docker-debian.sh 

### K3D ###
RUN /bin/bash /tmp/workspace/bin/install_scripts/k3d-debian.sh

### Kubectl and Helm ###
RUN /bin/bash /tmp/workspace/bin/install_scripts/kubectl-helm-debian.sh

# ### Rust ###
# ENV PATH="/usr/local/cargo/bin:$PATH"
# RUN /bin/bash /tmp/workspace/bin/install_scripts/rust-debian.sh && \
#     rustup default stable && \
#     cargo install wasm-pack


### Java ###
ENV LANG="en_US.UTF-8"

### SDK Man ###

# Switch from `sh -c` to `bash -c` as the shell behind a `RUN` command.
SHELL ["/bin/bash", "-c"]

# Install sdkman
USER ${HOST_USERNAME}
ENV HOME /home/${HOST_USERNAME}
RUN curl -s "https://get.sdkman.io" | bash

# Install Java and maven
RUN source "$HOME/.sdkman/bin/sdkman-init.sh" \
    && sdk install java 11.0.12-open \
    && sdk install maven 3.8.2

# Update environment variables with the new user's paths
ENV PATH=${HOME}/.sdkman/candidates/java/current/bin:$PATH
ENV PATH=${HOME}/.sdkman/candidates/scala/current/bin:$PATH
ENV PATH=${HOME}/.sdkman/candidates/sbt/current/bin:$PATH
ENV PATH=${HOME}/.sdkman/candidates/maven/current/bin:$PATH

# Switch back to root user
USER root


### Node.js ###
ENV NVM_DIR="/usr/local/share/nvm"
ENV NVM_SYMLINK_CURRENT=true \
    PATH=${NVM_DIR}/current/bin:${PATH}
RUN apt-get update && /bin/bash /tmp/workspace/bin/install_scripts/node-debian.sh "${NVM_DIR}" "$NODE_VERSION"


# Update npm, install yarn globally
# RUN npm install -g npm yarn

### Lake FS CLI ###
RUN /bin/bash /tmp/workspace/bin/install_scripts/lakefs-debian.sh "${LAKEFS_VERSION}"

### Python ###

# Install python dependencies
RUN python -m pip install --upgrade pip setuptools wheel
COPY ./requirements.txt /tmp/workspace/requirements.txt
COPY --from=dependencies /tmp/python_packages /tmp/python_packages
RUN chown $HOST_USER_ID:$HOST_GROUP_ID -R /tmp
USER ${HOST_USERNAME}
RUN python -m pip install --no-warn-script-location --no-deps --no-index --find-links file:///tmp/python_packages -r /tmp/workspace/requirements.txt
# Switch back to root user
USER root
# Install PySpark and its dependencies (py4j) which are bundled with Spark
RUN cd /opt/spark/python \
    && python -m pip install --no-warn-script-location --no-deps --no-index --find-links file:///tmp/python_packages py4j \
    && python -m pip install .


### Cleanup ###
USER root
RUN rm -rf /tmp/workspace
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
USER ${HOST_USERNAME}
WORKDIR /workspace


# Install CPU Deep Learning Frameworks
RUN pip install --upgrade "jax[cpu]"
RUN pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu

# Required for Docker-from-Docker
ENTRYPOINT ["/usr/local/share/docker-init.sh"]
CMD ["sleep", "infinity"]


### CICD Container ###
# Contains everything required to build project artifacts and run tests





# ARG CUDA_IMAGE="12.1.0-base-ubuntu22.04"
# ARG CUDA_OS="ubuntu22.04"


# FROM devcontainer as gpu_devcontainer

# # Install PyTorch
# #NOTE As of 2023-06-03 the nightly build is the only way to get PyTorch to work with CUDA 12.1
# RUN pip install --pre torch torchvision torchaudio --index-url https://download.pytorch.org/whl/nightly/cu121

# # Install the CUDA version of Jax
# # RUN pip install --upgrade "jax[cuda]" -f https://storage.googleapis.com/jax-releases/jax_cuda_releases.html

# # CUDA 12 installation
# # Note: wheels only available on linux.
# #TODO there is a issue with the mismatch of the CUDA version of the host (whatever is latest) and the cuDNN version shipped with Jax via wheels on Pip.
# # Running pip install jax[cuda_local] will use the host's versions, but that is crashing as of 2021-06-03...
# RUN pip install --upgrade "jax[cuda12_pip]" -f https://storage.googleapis.com/jax-releases/jax_cuda_releases.html
# #https://storage.googleapis.com/jax-releases/cuda12/jaxlib-0.4.10+cuda12.cudnn88-cp39-cp39-manylinux2014_x86_64.whl
# #https://storage.googleapis.com/jax-releases/jax_cuda_releases.html


# CMD ["sleep", "infinity"]



FROM devcontainer as cicd_container

WORKDIR /workspace

# Required for Docker-from-Docker
ENTRYPOINT ["/usr/local/share/docker-init.sh"]
# Default command, overwritten for CI
CMD ["sleep", "infinity"]
