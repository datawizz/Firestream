#!/bin/bash

# Default output directory for the downloaded JARs
default_output_directory="/opt/maven/jars"

# Use the first argument as the output directory if provided, else use the default
output_directory=${1:-$default_output_directory}

# Create the output directory if it doesn't exist
mkdir -p "${output_directory}"

# List the artifacts to download
# Format: groupId:artifactId:version
artifacts=(
    "com.lucidworks.spark:spark-solr:3.9.0"
    "org.apache.spark:spark-core_2.12:3.1.2"
    "org.apache.spark:spark-sql_2.12:3.1.2"
    "org.apache.hadoop:hadoop-aws:3.3.1"
    "org.apache.hadoop:hadoop-common:3.3.4"
    "org.apache.spark:spark-hadoop-cloud_2.12:3.3.1"
    "org.apache.spark:spark-sql-kafka-0-10_2.12:3.3.1"
    "org.apache.spark:spark-streaming-kafka-0-10_2.12:3.3.1"
    "org.apache.kafka:kafka-clients:3.3.1"
    "org.apache.spark:spark-avro_2.12:3.2.1"
    "io.delta:delta-core_2.12:2.2.0"
    "org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:1.2.0"
    "org.projectnessie.nessie-integrations:nessie-spark-extensions-3.3_2.12:0.58.1"
)

# Download artifacts and their dependencies
for artifact in "${artifacts[@]}"; do
    IFS=':' read -ra artifact_parts <<< "$artifact"
    group_id="${artifact_parts[0]}"
    artifact_id="${artifact_parts[1]}"
    version="${artifact_parts[2]}"

    # Create a temporary directory for the artifact
    temp_dir=$(mktemp -d)

    # Generate a temporary pom.xml file for the artifact
    cat > "${temp_dir}/pom.xml" <<- EOM
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>temp</artifactId>
    <version>1.0.0</version>
    <dependencies>
        <dependency>
            <groupId>${group_id}</groupId>
            <artifactId>${artifact_id}</artifactId>
            <version>${version}</version>
        </dependency>
    </dependencies>
</project>
EOM

    # Download the artifact and its dependencies using the temporary pom.xml file
    mvn -f "${temp_dir}/pom.xml" dependency:copy-dependencies -DoutputDirectory="${output_directory}"

    # Remove the temporary directory
    rm -rf "${temp_dir}"
done


# jars_dir="/workspace/opt/maven/jars"
# jars=$(find "$jars_dir" -name '*.jar' | tr '\n' ':')
# export SPARK_EXTRA_CLASSPATH="$jars"