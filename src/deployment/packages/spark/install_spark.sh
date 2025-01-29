#!/bin/bash

# Default versions and installation paths
SPARK_VERSION="3.4.4"
SPARK_SHA512="405f604532c11c3793b757aae41bbae0f86f3bcab65f47dea07be9003532c382429feab67d8ed19721ee539801842b21d32685c37a97372f96d50b318f5b1bb0"
SPARK_HOME="/opt/spark"

HADOOP_VERSION="3.3.6"
HADOOP_SHA512="de3eaca2e0517e4b569a88b63c89fae19cb8ac6c01ff990f1ff8f0cc0f3128c8e8a23db01577ca562a0e0bb1b4a3889f8c74384e609cd55e537aada3dcaa9f8a"
HADOOP_HOME="/opt/hadoop"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to verify SHA512 hash
verify_sha512() {
    local file=$1
    local expected_hash=$2
    local computed_hash=$(sha512sum "$file" | awk '{print $1}')

    if [ "$computed_hash" != "$expected_hash" ]; then
        echo "Hash verification failed for $file"
        echo "Expected: $expected_hash"
        echo "Got: $computed_hash"
        return 1
    fi
    echo "Hash verification passed for $file"
    return 0
}

# Install dependencies
echo "Installing dependencies..."
apt-get update
apt-get install -y --no-install-recommends \
    wget \
    unzip \
    curl \
    ca-certificates \
    bash \
    tini \
    libc6 \
    libpam-modules \
    libnss3 \
    python3 \
    python3-pip

# Create necessary directories
mkdir -p "${SPARK_HOME}"
mkdir -p "${HADOOP_HOME}"
mkdir -p /tmp/downloads

# Change to downloads directory
cd /tmp/downloads

# Download and install Spark
echo "Downloading Apache Spark ${SPARK_VERSION}..."
wget "https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/spark-${SPARK_VERSION}-bin-without-hadoop.tgz"

echo "Verifying Spark download..."
verify_sha512 "spark-${SPARK_VERSION}-bin-without-hadoop.tgz" "${SPARK_SHA512}"

echo "Extracting Spark..."
tar xf "spark-${SPARK_VERSION}-bin-without-hadoop.tgz"
mv "spark-${SPARK_VERSION}-bin-without-hadoop"/* "${SPARK_HOME}"

# Download and install Hadoop
echo "Downloading Apache Hadoop ${HADOOP_VERSION}..."
wget "https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/hadoop-${HADOOP_VERSION}.tar.gz"

echo "Verifying Hadoop download..."
verify_sha512 "hadoop-${HADOOP_VERSION}.tar.gz" "${HADOOP_SHA512}"

echo "Extracting Hadoop..."
tar xf "hadoop-${HADOOP_VERSION}.tar.gz"
mv "hadoop-${HADOOP_VERSION}"/* "${HADOOP_HOME}"

# Configure Hadoop logging to WARN level
echo "Configuring Hadoop logging..."
sed -i 's/hadoop.root.logger=INFO,console/hadoop.root.logger=WARN,console/g' "${HADOOP_HOME}/etc/hadoop/log4j.properties"

# Set up environment variables
echo "Setting up environment variables..."
cat >> /etc/profile.d/spark-hadoop.sh << 'EOF'
export SPARK_HOME=/opt/spark
export HADOOP_HOME=/opt/hadoop
export PATH=$PATH:$SPARK_HOME/bin:$HADOOP_HOME/bin
export SPARK_DIST_CLASSPATH=/opt/hadoop/etc/hadoop:/opt/hadoop/share/hadoop/common/lib/*:/opt/hadoop/share/hadoop/common/*:/opt/hadoop/share/hadoop/hdfs:/opt/hadoop/share/hadoop/hdfs/lib/*:/opt/hadoop/share/hadoop/hdfs/*:/opt/hadoop/share/hadoop/mapreduce/lib/*:/opt/hadoop/share/hadoop/mapreduce/*:/opt/hadoop/share/hadoop/yarn:/opt/hadoop/share/hadoop/yarn/lib/*:/opt/hadoop/share/hadoop/yarn/*
export LD_LIBRARY_PATH=$HADOOP_HOME/lib/native:$LD_LIBRARY_PATH
EOF

# Clean up
echo "Cleaning up..."
cd /
rm -rf /tmp/downloads
apt-get clean
rm -rf /var/lib/apt/lists/*

echo "Installation complete!"
echo "Please source the environment variables:"
echo "source /etc/profile.d/spark-hadoop.sh"
