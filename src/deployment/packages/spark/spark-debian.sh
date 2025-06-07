#!/bin/bash

# Exit on any error
set -e

# https://dlcdn.apache.org/spark/
# https://dlcdn.apache.org/hadoop/common/
# https://dlcdn.apache.org/hadoop/common/hadoop-3.3.6/hadoop-3.3.6-aarch64.tar.gz.sha512

# Default versions and installation paths
SPARK_VERSION=${1:-"3.4.4"}
HADOOP_VERSION=${2:-"3.3.6"}
SPARK_HOME=${3:-"/opt/spark"}
HADOOP_HOME=${4:-"/opt/hadoop"}

# SHA512 checksums for verification
SPARK_SHA512="405f604532c11c3793b757aae41bbae0f86f3bcab65f47dea07be9003532c382429feab67d8ed19721ee539801842b21d32685c37a97372f96d50b318f5b1bb0"
HADOOP_SHA512="de3eaca2e0517e4b569a88b63c89fae19cb8ac6c01ff990f1ff8f0cc0f3128c8e8a23db01577ca562a0e0bb1b4a3889f8c74384e609cd55e537aada3dcaa9f8a"

echo "Installing Spark ${SPARK_VERSION} and Hadoop ${HADOOP_VERSION}"

# Create temporary directory for downloads
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Download and verify Spark
echo "Downloading Spark..."
wget "https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz"
echo "${SPARK_SHA512}  spark-${SPARK_VERSION}-bin-without-hadoop.tgz" | sha512sum -c - || {
    echo "Spark checksum verification failed"
    exit 1
}

# Download and verify Hadoop
echo "Downloading Hadoop..."
wget "https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz"
echo "${HADOOP_SHA512}  hadoop-${HADOOP_VERSION}.tar.gz" | sha512sum -c - || {
    echo "Hadoop checksum verification failed"
    exit 1
}

# Extract and install Spark
echo "Installing Spark..."
mkdir -p "${SPARK_HOME}"
tar xf "spark-${SPARK_VERSION}-bin-without-hadoop.tgz"
mv "spark-${SPARK_VERSION}-bin-without-hadoop"/* "${SPARK_HOME}"

# Extract and install Hadoop
echo "Installing Hadoop..."
mkdir -p "${HADOOP_HOME}"
tar xf "hadoop-${HADOOP_VERSION}.tar.gz"
mv "hadoop-${HADOOP_VERSION}"/* "${HADOOP_HOME}"

# Configure Hadoop logging
echo "Configuring Hadoop logging..."
sed -i 's/hadoop.root.logger=INFO,console/hadoop.root.logger=WARN,console/g' "${HADOOP_HOME}/etc/hadoop/log4j.properties"

# Set up environment variables
echo "Setting up environment variables..."
cat >> /etc/profile.d/spark-hadoop.sh << EOF
export SPARK_HOME=${SPARK_HOME}
export HADOOP_HOME=${HADOOP_HOME}
export PATH=\$PATH:\$SPARK_HOME/bin:\$HADOOP_HOME/bin
export SPARK_DIST_CLASSPATH=\$HADOOP_HOME/etc/hadoop:\$HADOOP_HOME/share/hadoop/common/lib/*:\$HADOOP_HOME/share/hadoop/common/*:\$HADOOP_HOME/share/hadoop/hdfs:\$HADOOP_HOME/share/hadoop/hdfs/lib/*:\$HADOOP_HOME/share/hadoop/hdfs/*:\$HADOOP_HOME/share/hadoop/mapreduce/lib/*:\$HADOOP_HOME/share/hadoop/mapreduce/*:\$HADOOP_HOME/share/hadoop/yarn:\$HADOOP_HOME/share/hadoop/yarn/lib/*:\$HADOOP_HOME/share/hadoop/yarn/*
export LD_LIBRARY_PATH=\$HADOOP_HOME/lib/native:\$LD_LIBRARY_PATH
EOF

# Clean up
echo "Cleaning up..."
cd
rm -rf "$TEMP_DIR"

echo "Installation complete!"
echo "Spark installed at: ${SPARK_HOME}"
echo "Hadoop installed at: ${HADOOP_HOME}"
echo "Environment variables have been set in /etc/profile.d/spark-hadoop.sh"
