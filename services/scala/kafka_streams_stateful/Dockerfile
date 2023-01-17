

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



# FROM openjdk:11-jre-slim-bullseye

# ## Install Scala Build Tools
# RUN /bin/bash /tmp/library-scripts/workspace/bin/sbt-debian.sh

# # Install sbt
# RUN apt-get update && apt-get install -y sbt

# Install additional OS packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    build-essential \
    gnupg \
    apt-utils


# Copy install scripts to image
COPY ./bin /tmp/library-scripts/
RUN /bin/bash /tmp/library-scripts/sbt-debian.sh


ADD scala/kafka_streams_stateful /app
WORKDIR /app
RUN sbt assembly


# CMD ["java", "-jar", "target/example.jar"]