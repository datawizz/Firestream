#!/bin/bash
set -e

### Hive MetaStore ###

# Create the directory if it doesn't exist
mkdir -p ${METASTORE_HOME}/conf

# Generate the metastore-site.xml with the substituted variables
cat << EOF > ${METASTORE_HOME}/conf/metastore-site.xml
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<?xml-stylesheet type="text/xsl" href="configuration.xsl"?>
<configuration>
  <property>
    <name>metastore.thrift.uris</name>
    <value>thrift://0.0.0.0:9083</value>
    <description>Thrift URI for the remote metastore. Used by metastore client to connect to remote metastore.</description>
  </property>
  <property>
    <name>metastore.task.threads.always</name>
    <value>org.apache.hadoop.hive.metastore.events.EventCleanerTask,org.apache.hadoop.hive.metastore.MaterializationsCacheCleanerTask</value>
  </property>
  <property>
    <name>metastore.expression.proxy</name>
    <value>org.apache.hadoop.hive.metastore.DefaultPartitionExpressionProxy</value>
  </property>
  <property>
    <name>javax.jdo.option.ConnectionURL</name>
    <value>${JDBC_CONNECTION_STRING}?createDatabaseIfNotExist=true</value>
  </property>
  <property>
    <name>javax.jdo.option.ConnectionDriverName</name>
    <value>org.postgresql.Driver</value>
  </property>
  <property>
    <name>javax.jdo.option.ConnectionUserName</name>
    <value>${POSTGRES_USER}</value>
  </property>
  <property>
    <name>javax.jdo.option.ConnectionPassword</name>
    <value>${POSTGRES_PASSWORD}</value>
  </property>
  <property>
    <name>datanucleus.autoCreateSchema</name>
    <value>true</value>
  </property>
  <property>
    <name>datanucleus.fixedDatastore</name>
    <value>false</value>
  </property>
  <property>
    <name>hive.metastore.schema.verification</name>
    <value>false</value>
  </property>
  <property>
    <name>hive.metastore.event.db.notification.api.auth</name>
    <value>false</value>
  </property>

  <property>
    <name>fs.s3a.access.key</name>
    <value>${S3_ACCESS_KEY_ID}</value>
  </property>
  <property>
    <name>fs.s3a.secret.key</name>
    <value>${S3_SECRET_ACCESS_KEY}</value>
  </property>
  <property>
    <name>fs.s3a.endpoint</name>
    <value>${S3_ENDPOINT_URL}</value>
  </property>
  <property>
    <name>fs.s3a.region</name>
    <value>${S3_DEFAULT_REGION}</value>
  </property>
  <property>
    <name>fs.s3a.path.style.access</name>
    <value>true</value>
  </property>
  <property>
    <name>fs.s3a.connection.ssl.enabled</name>
    <value>false</value>
  </property>
  <property>
    <name>hive.input.format</name>
    <value>io.delta.hive.HiveInputFormat</value>
  </property>

  <property>
      <name>hive.tez.input.format</name>
      <value>io.delta.hive.HiveInputFormat</value>
  </property>

</configuration>
EOF

# Generate the hive-site.xml with the substituted variables
cat << EOF > ${METASTORE_HOME}/conf/hive-site.xml
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<?xml-stylesheet type="text/xsl" href="configuration.xsl"?>
<configuration>
  <!-- Add your desired properties for hive-site.xml here -->
  <property>
    <name>hive.metastore.warehouse.dir</name>
    <value>/apps/hive/warehouse</value>
    <description>location of default database for the warehouse</description>
  </property>
  <property>
    <name>hive.metastore.uris</name>
    <value>thrift://0.0.0.0:9083</value>
  </property>
  <property>
    <name>hive.server2.thrift.port</name>
    <value>10000</value>
  </property>
  <property>
    <name>hive.server2.transport.mode</name>
    <value>binary</value>
  </property>
  <property>
    <name>datanucleus.autoStartMechanismMode</name>
    <value>ignored</value>
  </property>
</configuration>
EOF




# Initialize the metastore schema if not present
if [ ! -f "${METASTORE_HOME}/initialized" ]; then
  ${METASTORE_HOME}/bin/schematool -dbType "postgres" -initSchema -verbose
  touch "${METASTORE_HOME}/initialized"
fi

# Start the metastore
exec /opt/hive-metastore/bin/start-metastore
