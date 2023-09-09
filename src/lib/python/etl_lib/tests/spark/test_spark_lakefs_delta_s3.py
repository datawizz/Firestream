import os
import pytest
from datetime import datetime
import random
import string
from pyspark.sql.types import StringType, IntegerType, FloatType, StructField, StructType, TimestampType
from etl_lib.services.spark.client import SparkClient

LAKEFS_PATH = "s3a://fireworks/main/test2.parquet"

#TODO "lakefs" extension fails with error "WARN fs.FileSystem: Failed to initialize fileystem lakefs://fireworks/main/test2.parquet: java.io.IOException: Failed to get lakeFS blockstore type"
# LAKEFS_PATH = "lakefs://fireworks/main/test2.parquet"


def get_random_string(length):
    charset = string.ascii_lowercase
    return "".join(random.choice(charset) for _ in range(length))

def create_sample_data(num_rows):
    now_datetime = datetime.now()
    data = [{"yyyy_mm_dd": now_datetime.strftime("%Y-%m-%d"),
             "hh_mm": now_datetime.strftime("%H-%M"),
             "i": i+1,
             "datetime_now": now_datetime,
             "random_text": get_random_string(5),
             "random_float": round(random.uniform(0.01, 10000.01), 2)}
            for i in range(num_rows)]
    return data

def create_sample_df(spark_client):
    data = create_sample_data(100)



    schema = StructType([
        StructField("yyyy_mm_dd", StringType(), True),
        StructField("hh_mm", StringType(), True),
        StructField("i", IntegerType(), True),
        StructField("datetime_now", TimestampType(), True),
        StructField("random_text", StringType(), True),
        StructField("random_float", FloatType(), True),
    ])

    df = spark_client.spark_session.createDataFrame(data, schema=schema)
    return df


def test_write(spark_client):
    df = create_sample_df(spark_client)
    df.write.format("delta").mode("overwrite").option("path", LAKEFS_PATH).save()
    print("Wrote to lakefs")

def test_read(spark_client):
    df = spark_client.spark_session.read.format("delta").load(LAKEFS_PATH)
    df.show()
    print("Read from lakefs")

if __name__ == "__main__":
    client = SparkClient(app_name="Test_LakeFS", config={})
    test_write(client)
    test_read(client)
    client.stop()




# fireworks ➜ /workspace (FIRE-185-Spark-LakeFS-Delta-S3 ✗) $ python /workspace/src/lib/python/etl_lib/tests/spark/test_spark_lakefs_delta_s3.py
# Directory s3a://fireworks/spark_logs/ exists.
# org.apache.spark:spark-sql-kafka-0-10_2.12:3.4.1,org.apache.spark:spark-streaming-kafka-0-10_2.12:3.4.1,org.apache.kafka:kafka-clients:3.4.1,org.apache.hadoop:hadoop-aws:3.3.1,org.apache.hadoop:hadoop-common:3.3.1,org.apache.spark:spark-hadoop-cloud_2.12:3.4.1,io.delta:delta-core_2.12:2.4.0,io.lakefs:hadoop-lakefs-assembly:0.1.15
# 2023-09-02 20:22:02,227 WARN util.Utils: Your hostname, fireworks-dev resolves to a loopback address: 127.0.1.1; using 10.0.0.215 instead (on interface enp6s18)
# 2023-09-02 20:22:02,228 WARN util.Utils: Set SPARK_LOCAL_IP if you need to bind to another address
# :: loading settings :: url = jar:file:/opt/spark/jars/ivy-2.5.1.jar!/org/apache/ivy/core/settings/ivysettings.xml
# Ivy Default Cache set to: /home/fireworks/.ivy2/cache
# The jars for the packages stored in: /home/fireworks/.ivy2/jars
# org.apache.spark#spark-sql-kafka-0-10_2.12 added as a dependency
# org.apache.spark#spark-streaming-kafka-0-10_2.12 added as a dependency
# org.apache.kafka#kafka-clients added as a dependency
# org.apache.hadoop#hadoop-aws added as a dependency
# org.apache.hadoop#hadoop-common added as a dependency
# org.apache.spark#spark-hadoop-cloud_2.12 added as a dependency
# io.delta#delta-core_2.12 added as a dependency
# io.lakefs#hadoop-lakefs-assembly added as a dependency
# :: resolving dependencies :: org.apache.spark#spark-submit-parent-05685c8a-755b-45ff-8744-4d577b999626;1.0
#         confs: [default]
#         found org.apache.spark#spark-sql-kafka-0-10_2.12;3.4.1 in central
#         found org.apache.spark#spark-token-provider-kafka-0-10_2.12;3.4.1 in central
#         found org.apache.hadoop#hadoop-client-runtime;3.3.4 in central
#         found org.apache.hadoop#hadoop-client-api;3.3.4 in central
#         found org.xerial.snappy#snappy-java;1.1.10.1 in central
#         found org.slf4j#slf4j-api;2.0.6 in central
#         found commons-logging#commons-logging;1.1.3 in central
#         found com.google.code.findbugs#jsr305;3.0.0 in central
#         found org.apache.commons#commons-pool2;2.11.1 in central
#         found org.apache.spark#spark-streaming-kafka-0-10_2.12;3.4.1 in central
#         found org.apache.kafka#kafka-clients;3.4.1 in central
#         found com.github.luben#zstd-jni;1.5.2-1 in central
#         found org.lz4#lz4-java;1.8.0 in central
#         found org.apache.hadoop#hadoop-aws;3.3.1 in central
#         found com.amazonaws#aws-java-sdk-bundle;1.11.901 in central
#         found org.wildfly.openssl#wildfly-openssl;1.0.7.Final in central
#         found org.apache.hadoop#hadoop-common;3.3.1 in central
#         found org.apache.hadoop.thirdparty#hadoop-shaded-protobuf_3_7;1.1.1 in central
#         found org.apache.hadoop#hadoop-annotations;3.3.1 in central
#         found org.apache.hadoop.thirdparty#hadoop-shaded-guava;1.1.1 in central
#         found com.google.guava#guava;27.0-jre in central
#         found com.google.guava#failureaccess;1.0 in central
#         found com.google.guava#listenablefuture;9999.0-empty-to-avoid-conflict-with-guava in central
#         found com.google.code.findbugs#jsr305;3.0.2 in central
#         found org.checkerframework#checker-qual;2.5.2 in central
#         found com.google.j2objc#j2objc-annotations;1.1 in central
#         found org.codehaus.mojo#animal-sniffer-annotations;1.17 in central
#         found commons-cli#commons-cli;1.2 in central
#         found org.apache.commons#commons-math3;3.1.1 in central
#         found org.apache.httpcomponents#httpclient;4.5.13 in central
#         found org.apache.httpcomponents#httpcore;4.4.13 in central
#         found commons-codec#commons-codec;1.11 in central
#         found commons-io#commons-io;2.8.0 in central
#         found commons-net#commons-net;3.6 in central
#         found commons-collections#commons-collections;3.2.2 in central
#         found javax.servlet#javax.servlet-api;3.1.0 in central
#         found org.eclipse.jetty#jetty-server;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-http;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-util;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-io;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-servlet;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-security;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-util-ajax;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-webapp;9.4.40.v20210413 in central
#         found org.eclipse.jetty#jetty-xml;9.4.40.v20210413 in central
#         found com.sun.jersey#jersey-core;1.19 in central
#         found javax.ws.rs#jsr311-api;1.1.1 in central
#         found com.sun.jersey#jersey-servlet;1.19 in central
#         found com.sun.jersey#jersey-server;1.19 in central
#         found com.sun.jersey#jersey-json;1.19 in central
#         found org.codehaus.jettison#jettison;1.1 in central
#         found com.sun.xml.bind#jaxb-impl;2.2.3-1 in central
#         found javax.xml.bind#jaxb-api;2.2.11 in central
#         found org.codehaus.jackson#jackson-core-asl;1.9.13 in central
#         found org.codehaus.jackson#jackson-mapper-asl;1.9.13 in central
#         found org.codehaus.jackson#jackson-jaxrs;1.9.13 in central
#         found org.codehaus.jackson#jackson-xc;1.9.13 in central
#         found log4j#log4j;1.2.17 in central
#         found commons-beanutils#commons-beanutils;1.9.4 in central
#         found org.apache.commons#commons-configuration2;2.1.1 in central
#         found org.apache.commons#commons-lang3;3.7 in central
#         found org.apache.commons#commons-text;1.4 in central
#         found org.slf4j#slf4j-log4j12;1.7.30 in central
#         found org.apache.avro#avro;1.7.7 in central
#         found com.thoughtworks.paranamer#paranamer;2.3 in central
#         found org.apache.commons#commons-compress;1.19 in central
#         found com.google.re2j#re2j;1.1 in central
#         found com.google.protobuf#protobuf-java;2.5.0 in central
#         found com.google.code.gson#gson;2.2.4 in central
#         found org.apache.hadoop#hadoop-auth;3.3.1 in central
#         found com.nimbusds#nimbus-jose-jwt;9.8.1 in central
#         found com.github.stephenc.jcip#jcip-annotations;1.0-1 in central
#         found net.minidev#json-smart;2.4.2 in central
#         found net.minidev#accessors-smart;2.4.2 in central
#         found org.ow2.asm#asm;5.0.4 in central
#         found org.apache.zookeeper#zookeeper;3.5.6 in central
#         found org.apache.zookeeper#zookeeper-jute;3.5.6 in central
#         found org.apache.yetus#audience-annotations;0.5.0 in central
#         found org.apache.curator#curator-framework;4.2.0 in central
#         found org.apache.curator#curator-client;4.2.0 in central
#         found org.apache.kerby#kerb-simplekdc;1.0.1 in central
#         found org.apache.kerby#kerb-client;1.0.1 in central
#         found org.apache.kerby#kerby-config;1.0.1 in central
#         found org.apache.kerby#kerb-core;1.0.1 in central
#         found org.apache.kerby#kerby-pkix;1.0.1 in central
#         found org.apache.kerby#kerby-asn1;1.0.1 in central
#         found org.apache.kerby#kerby-util;1.0.1 in central
#         found org.apache.kerby#kerb-common;1.0.1 in central
#         found org.apache.kerby#kerb-crypto;1.0.1 in central
#         found org.apache.kerby#kerb-util;1.0.1 in central
#         found org.apache.kerby#token-provider;1.0.1 in central
#         found org.apache.kerby#kerb-admin;1.0.1 in central
#         found org.apache.kerby#kerb-server;1.0.1 in central
#         found org.apache.kerby#kerb-identity;1.0.1 in central
#         found org.apache.kerby#kerby-xdr;1.0.1 in central
#         found com.jcraft#jsch;0.1.55 in central
#         found org.apache.curator#curator-recipes;4.2.0 in central
#         found org.apache.htrace#htrace-core4;4.1.0-incubating in central
#         found com.fasterxml.jackson.core#jackson-databind;2.10.5.1 in central
#         found com.fasterxml.jackson.core#jackson-annotations;2.10.5 in central
#         found com.fasterxml.jackson.core#jackson-core;2.10.5 in central
#         found org.codehaus.woodstox#stax2-api;4.2.1 in central
#         found com.fasterxml.woodstox#woodstox-core;5.3.0 in central
#         found dnsjava#dnsjava;2.1.7 in central
#         found jakarta.activation#jakarta.activation-api;1.2.1 in central
#         found javax.servlet.jsp#jsp-api;2.1 in central
#         found org.apache.spark#spark-hadoop-cloud_2.12;3.4.1 in central
#         found org.apache.hadoop#hadoop-aws;3.3.4 in central
#         found com.amazonaws#aws-java-sdk-bundle;1.12.262 in central
#         found org.apache.hadoop#hadoop-openstack;3.3.4 in central
#         found org.apache.hadoop#hadoop-annotations;3.3.4 in central
#         found org.apache.httpcomponents#httpcore;4.4.16 in central
#         found com.fasterxml.jackson.core#jackson-annotations;2.14.2 in central
#         found com.fasterxml.jackson.core#jackson-databind;2.14.2 in central
#         found com.fasterxml.jackson.core#jackson-core;2.14.2 in central
#         found com.google.cloud.bigdataoss#gcs-connector;hadoop3-2.2.11 in central
#         found joda-time#joda-time;2.12.2 in central
#         found com.fasterxml.jackson.dataformat#jackson-dataformat-cbor;2.14.2 in central
#         found org.apache.httpcomponents#httpclient;4.5.14 in central
#         found commons-codec#commons-codec;1.15 in central
#         found org.apache.hadoop#hadoop-azure;3.3.4 in central
#         found com.microsoft.azure#azure-storage;7.0.1 in central
#         found com.microsoft.azure#azure-keyvault-core;1.0.0 in central
#         found org.apache.hadoop#hadoop-cloud-storage;3.3.4 in central
#         found org.apache.hadoop#hadoop-aliyun;3.3.4 in central
#         found com.aliyun.oss#aliyun-sdk-oss;3.13.0 in central
#         found org.jdom#jdom2;2.0.6 in central
#         found com.aliyun#aliyun-java-sdk-core;4.5.10 in central
#         found com.google.code.gson#gson;2.8.9 in central
#         found org.ini4j#ini4j;0.5.4 in central
#         found io.opentracing#opentracing-api;0.33.0 in central
#         found io.opentracing#opentracing-util;0.33.0 in central
#         found io.opentracing#opentracing-noop;0.33.0 in central
#         found com.aliyun#aliyun-java-sdk-ram;3.1.0 in central
#         found com.aliyun#aliyun-java-sdk-kms;2.11.0 in central
#         found org.apache.hadoop#hadoop-azure-datalake;3.3.4 in central
#         found com.microsoft.azure#azure-data-lake-store-sdk;2.3.9 in central
#         found org.eclipse.jetty#jetty-util;9.4.50.v20221201 in central
#         found org.eclipse.jetty#jetty-util-ajax;9.4.50.v20221201 in central
#         found io.delta#delta-core_2.12;2.4.0 in central
#         found io.delta#delta-storage;2.4.0 in central
#         found org.antlr#antlr4-runtime;4.9.3 in central
#         found io.lakefs#hadoop-lakefs-assembly;0.1.15 in central
# :: resolution report :: resolve 2473ms :: artifacts dl 63ms
#         :: modules in use:
#         com.aliyun#aliyun-java-sdk-core;4.5.10 from central in [default]
#         com.aliyun#aliyun-java-sdk-kms;2.11.0 from central in [default]
#         com.aliyun#aliyun-java-sdk-ram;3.1.0 from central in [default]
#         com.aliyun.oss#aliyun-sdk-oss;3.13.0 from central in [default]
#         com.amazonaws#aws-java-sdk-bundle;1.12.262 from central in [default]
#         com.fasterxml.jackson.core#jackson-annotations;2.14.2 from central in [default]
#         com.fasterxml.jackson.core#jackson-core;2.14.2 from central in [default]
#         com.fasterxml.jackson.core#jackson-databind;2.14.2 from central in [default]
#         com.fasterxml.jackson.dataformat#jackson-dataformat-cbor;2.14.2 from central in [default]
#         com.fasterxml.woodstox#woodstox-core;5.3.0 from central in [default]
#         com.github.luben#zstd-jni;1.5.2-1 from central in [default]
#         com.github.stephenc.jcip#jcip-annotations;1.0-1 from central in [default]
#         com.google.cloud.bigdataoss#gcs-connector;hadoop3-2.2.11 from central in [default]
#         com.google.code.findbugs#jsr305;3.0.2 from central in [default]
#         com.google.code.gson#gson;2.8.9 from central in [default]
#         com.google.guava#failureaccess;1.0 from central in [default]
#         com.google.guava#guava;27.0-jre from central in [default]
#         com.google.guava#listenablefuture;9999.0-empty-to-avoid-conflict-with-guava from central in [default]
#         com.google.j2objc#j2objc-annotations;1.1 from central in [default]
#         com.google.protobuf#protobuf-java;2.5.0 from central in [default]
#         com.google.re2j#re2j;1.1 from central in [default]
#         com.jcraft#jsch;0.1.55 from central in [default]
#         com.microsoft.azure#azure-data-lake-store-sdk;2.3.9 from central in [default]
#         com.microsoft.azure#azure-keyvault-core;1.0.0 from central in [default]
#         com.microsoft.azure#azure-storage;7.0.1 from central in [default]
#         com.nimbusds#nimbus-jose-jwt;9.8.1 from central in [default]
#         com.sun.jersey#jersey-core;1.19 from central in [default]
#         com.sun.jersey#jersey-json;1.19 from central in [default]
#         com.sun.jersey#jersey-server;1.19 from central in [default]
#         com.sun.jersey#jersey-servlet;1.19 from central in [default]
#         com.sun.xml.bind#jaxb-impl;2.2.3-1 from central in [default]
#         com.thoughtworks.paranamer#paranamer;2.3 from central in [default]
#         commons-beanutils#commons-beanutils;1.9.4 from central in [default]
#         commons-cli#commons-cli;1.2 from central in [default]
#         commons-codec#commons-codec;1.15 from central in [default]
#         commons-collections#commons-collections;3.2.2 from central in [default]
#         commons-io#commons-io;2.8.0 from central in [default]
#         commons-logging#commons-logging;1.1.3 from central in [default]
#         commons-net#commons-net;3.6 from central in [default]
#         dnsjava#dnsjava;2.1.7 from central in [default]
#         io.delta#delta-core_2.12;2.4.0 from central in [default]
#         io.delta#delta-storage;2.4.0 from central in [default]
#         io.lakefs#hadoop-lakefs-assembly;0.1.15 from central in [default]
#         io.opentracing#opentracing-api;0.33.0 from central in [default]
#         io.opentracing#opentracing-noop;0.33.0 from central in [default]
#         io.opentracing#opentracing-util;0.33.0 from central in [default]
#         jakarta.activation#jakarta.activation-api;1.2.1 from central in [default]
#         javax.servlet#javax.servlet-api;3.1.0 from central in [default]
#         javax.servlet.jsp#jsp-api;2.1 from central in [default]
#         javax.ws.rs#jsr311-api;1.1.1 from central in [default]
#         javax.xml.bind#jaxb-api;2.2.11 from central in [default]
#         joda-time#joda-time;2.12.2 from central in [default]
#         log4j#log4j;1.2.17 from central in [default]
#         net.minidev#accessors-smart;2.4.2 from central in [default]
#         net.minidev#json-smart;2.4.2 from central in [default]
#         org.antlr#antlr4-runtime;4.9.3 from central in [default]
#         org.apache.avro#avro;1.7.7 from central in [default]
#         org.apache.commons#commons-compress;1.19 from central in [default]
#         org.apache.commons#commons-configuration2;2.1.1 from central in [default]
#         org.apache.commons#commons-lang3;3.7 from central in [default]
#         org.apache.commons#commons-math3;3.1.1 from central in [default]
#         org.apache.commons#commons-pool2;2.11.1 from central in [default]
#         org.apache.commons#commons-text;1.4 from central in [default]
#         org.apache.curator#curator-client;4.2.0 from central in [default]
#         org.apache.curator#curator-framework;4.2.0 from central in [default]
#         org.apache.curator#curator-recipes;4.2.0 from central in [default]
#         org.apache.hadoop#hadoop-aliyun;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-annotations;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-auth;3.3.1 from central in [default]
#         org.apache.hadoop#hadoop-aws;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-azure;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-azure-datalake;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-client-api;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-client-runtime;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-cloud-storage;3.3.4 from central in [default]
#         org.apache.hadoop#hadoop-common;3.3.1 from central in [default]
#         org.apache.hadoop#hadoop-openstack;3.3.4 from central in [default]
#         org.apache.hadoop.thirdparty#hadoop-shaded-guava;1.1.1 from central in [default]
#         org.apache.hadoop.thirdparty#hadoop-shaded-protobuf_3_7;1.1.1 from central in [default]
#         org.apache.htrace#htrace-core4;4.1.0-incubating from central in [default]
#         org.apache.httpcomponents#httpclient;4.5.14 from central in [default]
#         org.apache.httpcomponents#httpcore;4.4.16 from central in [default]
#         org.apache.kafka#kafka-clients;3.4.1 from central in [default]
#         org.apache.kerby#kerb-admin;1.0.1 from central in [default]
#         org.apache.kerby#kerb-client;1.0.1 from central in [default]
#         org.apache.kerby#kerb-common;1.0.1 from central in [default]
#         org.apache.kerby#kerb-core;1.0.1 from central in [default]
#         org.apache.kerby#kerb-crypto;1.0.1 from central in [default]
#         org.apache.kerby#kerb-identity;1.0.1 from central in [default]
#         org.apache.kerby#kerb-server;1.0.1 from central in [default]
#         org.apache.kerby#kerb-simplekdc;1.0.1 from central in [default]
#         org.apache.kerby#kerb-util;1.0.1 from central in [default]
#         org.apache.kerby#kerby-asn1;1.0.1 from central in [default]
#         org.apache.kerby#kerby-config;1.0.1 from central in [default]
#         org.apache.kerby#kerby-pkix;1.0.1 from central in [default]
#         org.apache.kerby#kerby-util;1.0.1 from central in [default]
#         org.apache.kerby#kerby-xdr;1.0.1 from central in [default]
#         org.apache.kerby#token-provider;1.0.1 from central in [default]
#         org.apache.spark#spark-hadoop-cloud_2.12;3.4.1 from central in [default]
#         org.apache.spark#spark-sql-kafka-0-10_2.12;3.4.1 from central in [default]
#         org.apache.spark#spark-streaming-kafka-0-10_2.12;3.4.1 from central in [default]
#         org.apache.spark#spark-token-provider-kafka-0-10_2.12;3.4.1 from central in [default]
#         org.apache.yetus#audience-annotations;0.5.0 from central in [default]
#         org.apache.zookeeper#zookeeper;3.5.6 from central in [default]
#         org.apache.zookeeper#zookeeper-jute;3.5.6 from central in [default]
#         org.checkerframework#checker-qual;2.5.2 from central in [default]
#         org.codehaus.jackson#jackson-core-asl;1.9.13 from central in [default]
#         org.codehaus.jackson#jackson-jaxrs;1.9.13 from central in [default]
#         org.codehaus.jackson#jackson-mapper-asl;1.9.13 from central in [default]
#         org.codehaus.jackson#jackson-xc;1.9.13 from central in [default]
#         org.codehaus.jettison#jettison;1.1 from central in [default]
#         org.codehaus.mojo#animal-sniffer-annotations;1.17 from central in [default]
#         org.codehaus.woodstox#stax2-api;4.2.1 from central in [default]
#         org.eclipse.jetty#jetty-http;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-io;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-security;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-server;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-servlet;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-util;9.4.50.v20221201 from central in [default]
#         org.eclipse.jetty#jetty-util-ajax;9.4.50.v20221201 from central in [default]
#         org.eclipse.jetty#jetty-webapp;9.4.40.v20210413 from central in [default]
#         org.eclipse.jetty#jetty-xml;9.4.40.v20210413 from central in [default]
#         org.ini4j#ini4j;0.5.4 from central in [default]
#         org.jdom#jdom2;2.0.6 from central in [default]
#         org.lz4#lz4-java;1.8.0 from central in [default]
#         org.ow2.asm#asm;5.0.4 from central in [default]
#         org.slf4j#slf4j-api;2.0.6 from central in [default]
#         org.slf4j#slf4j-log4j12;1.7.30 from central in [default]
#         org.wildfly.openssl#wildfly-openssl;1.0.7.Final from central in [default]
#         org.xerial.snappy#snappy-java;1.1.10.1 from central in [default]
#         :: evicted modules:
#         org.apache.kafka#kafka-clients;3.3.2 by [org.apache.kafka#kafka-clients;3.4.1] in [default]
#         com.google.code.findbugs#jsr305;3.0.0 by [com.google.code.findbugs#jsr305;3.0.2] in [default]
#         org.xerial.snappy#snappy-java;1.1.8.4 by [org.xerial.snappy#snappy-java;1.1.10.1] in [default]
#         org.slf4j#slf4j-api;1.7.36 by [org.slf4j#slf4j-api;2.0.6] in [default]
#         org.apache.hadoop#hadoop-aws;3.3.1 by [org.apache.hadoop#hadoop-aws;3.3.4] in [default]
#         com.amazonaws#aws-java-sdk-bundle;1.11.901 by [com.amazonaws#aws-java-sdk-bundle;1.12.262] in [default]
#         org.apache.hadoop#hadoop-annotations;3.3.1 by [org.apache.hadoop#hadoop-annotations;3.3.4] in [default]
#         org.apache.httpcomponents#httpclient;4.5.13 by [org.apache.httpcomponents#httpclient;4.5.14] in [default]
#         org.apache.httpcomponents#httpcore;4.4.13 by [org.apache.httpcomponents#httpcore;4.4.16] in [default]
#         commons-codec#commons-codec;1.11 by [commons-codec#commons-codec;1.15] in [default]
#         org.eclipse.jetty#jetty-util;9.4.40.v20210413 by [org.eclipse.jetty#jetty-util;9.4.50.v20221201] in [default]
#         org.eclipse.jetty#jetty-util-ajax;9.4.40.v20210413 by [org.eclipse.jetty#jetty-util-ajax;9.4.50.v20221201] in [default]
#         org.slf4j#slf4j-api;1.7.30 by [org.slf4j#slf4j-api;2.0.6] in [default]
#         org.xerial.snappy#snappy-java;1.1.8.2 by [org.xerial.snappy#snappy-java;1.1.10.1] in [default]
#         com.google.code.gson#gson;2.2.4 by [com.google.code.gson#gson;2.8.9] in [default]
#         com.fasterxml.jackson.core#jackson-databind;2.10.5.1 by [com.fasterxml.jackson.core#jackson-databind;2.14.2] in [default]
#         com.fasterxml.jackson.core#jackson-annotations;2.10.5 by [com.fasterxml.jackson.core#jackson-annotations;2.14.2] in [default]
#         com.fasterxml.jackson.core#jackson-core;2.10.5 by [com.fasterxml.jackson.core#jackson-core;2.14.2] in [default]
#         org.eclipse.jetty#jetty-util-ajax;9.4.43.v20210629 by [org.eclipse.jetty#jetty-util-ajax;9.4.50.v20221201] in [default]
#         ---------------------------------------------------------------------
#         |                  |            modules            ||   artifacts   |
#         |       conf       | number| search|dwnlded|evicted|| number|dwnlded|
#         ---------------------------------------------------------------------
#         |      default     |  149  |   0   |   0   |   19  ||  130  |   0   |
#         ---------------------------------------------------------------------
# :: retrieving :: org.apache.spark#spark-submit-parent-05685c8a-755b-45ff-8744-4d577b999626
#         confs: [default]
#         0 artifacts copied, 130 already retrieved (0kB/27ms)
# Wrote to lakefs                                                                 
# +----------+-----+---+--------------------+-----------+------------+
# |yyyy_mm_dd|hh_mm|  i|        datetime_now|random_text|random_float|
# +----------+-----+---+--------------------+-----------+------------+
# |2023-09-02|20-22| 61|2023-09-02 20:22:...|      imrsk|     5884.14|
# |2023-09-02|20-22| 62|2023-09-02 20:22:...|      orylx|     6391.24|
# |2023-09-02|20-22| 63|2023-09-02 20:22:...|      rqica|     4169.14|
# |2023-09-02|20-22| 64|2023-09-02 20:22:...|      xmejn|     9144.62|
# |2023-09-02|20-22| 65|2023-09-02 20:22:...|      uemgw|     9829.18|
# |2023-09-02|20-22| 66|2023-09-02 20:22:...|      pocue|     2719.02|
# |2023-09-02|20-22| 67|2023-09-02 20:22:...|      grjui|     9496.48|
# |2023-09-02|20-22| 68|2023-09-02 20:22:...|      ceudh|     5256.91|
# |2023-09-02|20-22| 69|2023-09-02 20:22:...|      bswae|     7631.15|
# |2023-09-02|20-22| 70|2023-09-02 20:22:...|      useoj|     1824.81|
# |2023-09-02|20-22| 51|2023-09-02 20:22:...|      msdwj|     2227.19|
# |2023-09-02|20-22| 52|2023-09-02 20:22:...|      onuwp|     4997.56|
# |2023-09-02|20-22| 53|2023-09-02 20:22:...|      uzqyu|     6630.18|
# |2023-09-02|20-22| 54|2023-09-02 20:22:...|      ivkon|     2309.68|
# |2023-09-02|20-22| 55|2023-09-02 20:22:...|      edcxa|     7431.97|
# |2023-09-02|20-22| 56|2023-09-02 20:22:...|      zefbc|     9794.35|
# |2023-09-02|20-22| 57|2023-09-02 20:22:...|      hbaji|     8841.66|
# |2023-09-02|20-22| 58|2023-09-02 20:22:...|      yrafe|     3481.95|
# |2023-09-02|20-22| 59|2023-09-02 20:22:...|      bgvwp|     7492.78|
# |2023-09-02|20-22| 60|2023-09-02 20:22:...|      bviup|       576.0|
# +----------+-----+---+--------------------+-----------+------------+
# only showing top 20 rows

# Read from lakefs