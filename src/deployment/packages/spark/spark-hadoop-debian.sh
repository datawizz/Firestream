#!/bin/bash
# Default versions and installation paths
SPARK_VERSION="3.4.4"
SPARK_SHA512="405f604532c11c3793b757aae41bbae0f86f3bcab65f47dea07be9003532c382429feab67d8ed19721ee539801842b21d32685c37a97372f96d50b318f5b1bb0"
SPARK_HOME="$HOME/opt/spark"
HADOOP_VERSION="3.3.6"
HADOOP_SHA512="de3eaca2e0517e4b569a88b63c89fae19cb8ac6c01ff990f1ff8f0cc0f3128c8e8a23db01577ca562a0e0bb1b4a3889f8c74384e609cd55e537aada3dcaa9f8a"
HADOOP_HOME="$HOME/opt/hadoop"

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

# Check for required commands
for cmd in wget tar sha512sum; do
    if ! command_exists "$cmd"; then
        echo "Required command '$cmd' is missing. Please install it first."
        exit 1
    fi
done

# Create necessary directories
mkdir -p "${SPARK_HOME}"
mkdir -p "${HADOOP_HOME}"
mkdir -p "$HOME/downloads"

# Change to downloads directory
cd "$HOME/downloads"

# Download and install Spark
SPARK_FILE="spark-${SPARK_VERSION}-bin-without-hadoop.tgz"
if [ -f "$SPARK_FILE" ]; then
    echo "Spark archive already exists, verifying..."
    if verify_sha512 "$SPARK_FILE" "${SPARK_SHA512}"; then
        echo "Using existing Spark archive..."
    else
        echo "Existing Spark archive is invalid, downloading..."
        wget "https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/${SPARK_FILE}"
        verify_sha512 "$SPARK_FILE" "${SPARK_SHA512}"
    fi
else
    echo "Downloading Apache Spark ${SPARK_VERSION}..."
    wget "https://dlcdn.apache.org/spark/spark-${SPARK_VERSION}/${SPARK_FILE}"
    verify_sha512 "$SPARK_FILE" "${SPARK_SHA512}"
fi

echo "Extracting Spark..."
tar xf "$SPARK_FILE"
# Clear destination directory first
rm -rf "${SPARK_HOME}"/*
mv "spark-${SPARK_VERSION}-bin-without-hadoop"/* "${SPARK_HOME}"

# Download and install Hadoop
HADOOP_FILE="hadoop-${HADOOP_VERSION}.tar.gz"
if [ -f "$HADOOP_FILE" ]; then
    echo "Hadoop archive already exists, verifying..."
    if verify_sha512 "$HADOOP_FILE" "${HADOOP_SHA512}"; then
        echo "Using existing Hadoop archive..."
    else
        echo "Existing Hadoop archive is invalid, downloading..."
        wget "https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/${HADOOP_FILE}"
        verify_sha512 "$HADOOP_FILE" "${HADOOP_SHA512}"
    fi
else
    echo "Downloading Apache Hadoop ${HADOOP_VERSION}..."
    wget "https://dlcdn.apache.org/hadoop/common/hadoop-${HADOOP_VERSION}/${HADOOP_FILE}"
    verify_sha512 "$HADOOP_FILE" "${HADOOP_SHA512}"
fi

echo "Extracting Hadoop..."
tar xf "$HADOOP_FILE"
rm -rf "${HADOOP_HOME}"/*
mv "hadoop-${HADOOP_VERSION}"/* "${HADOOP_HOME}"

# Configure Hadoop logging to WARN level
echo "Configuring Hadoop logging..."
if [ -f "${HADOOP_HOME}/etc/hadoop/log4j.properties" ]; then
    sed -i 's/hadoop.root.logger=INFO,console/hadoop.root.logger=WARN,console/g' "${HADOOP_HOME}/etc/hadoop/log4j.properties"
fi

# Set up environment variables in user's bashrc
echo "Setting up environment variables..."
cat >> "$HOME/.bashrc" << EOF

# Spark and Hadoop environment variables
export SPARK_HOME=$HOME/opt/spark
export HADOOP_HOME=$HOME/opt/hadoop
export PATH=\$PATH:\$SPARK_HOME/bin:\$HADOOP_HOME/bin
export SPARK_DIST_CLASSPATH=$HOME/opt/hadoop/etc/hadoop:$HOME/opt/hadoop/share/hadoop/common/lib/*:$HOME/opt/hadoop/share/hadoop/common/*:$HOME/opt/hadoop/share/hadoop/hdfs:$HOME/opt/hadoop/share/hadoop/hdfs/lib/*:$HOME/opt/hadoop/share/hadoop/hdfs/*:$HOME/opt/hadoop/share/hadoop/mapreduce/lib/*:$HOME/opt/hadoop/share/hadoop/mapreduce/*:$HOME/opt/hadoop/share/hadoop/yarn:$HOME/opt/hadoop/share/hadoop/yarn/lib/*:$HOME/opt/hadoop/share/hadoop/yarn/*
export LD_LIBRARY_PATH=\$HADOOP_HOME/lib/native:\$LD_LIBRARY_PATH
EOF

# Clean up
echo "Cleaning up..."
rm -rf "spark-${SPARK_VERSION}-bin-without-hadoop"
rm -rf "hadoop-${HADOOP_VERSION}"

echo "Installation complete!"
echo "Please source your ~/.bashrc or start a new terminal session:"
echo "source ~/.bashrc"
