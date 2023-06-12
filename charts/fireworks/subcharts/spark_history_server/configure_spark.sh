#!/bin/bash

# Set default values if environment variables are not provided
LOG_LEVEL=${LOG_LEVEL:-INFO}
S3_BUCKET_NAME=${S3_BUCKET_NAME:-your-bucket-name}
S3_ENDPOINT=${S3_ENDPOINT:-http://your-local-s3-api:port}
S3_ACCESS_KEY=${S3_ACCESS_KEY:-your-access-key}
S3_SECRET_KEY=${S3_SECRET_KEY:-your-secret-key}

# Create log4j.properties file
cat > $SPARK_HOME/conf/log4j.properties <<EOL
log4j.rootCategory=$LOG_LEVEL, console
log4j.appender.console=org.apache.log4j.ConsoleAppender
log4j.appender.console.target=System.err
log4j.appender.console.layout=org.apache.log4j.PatternLayout
log4j.appender.console.layout.ConversionPattern=%d{yy/MM/dd HH:mm:ss} %p %c{1}: %m%n
EOL

# Create spark-defaults.conf file
cat > $SPARK_HOME/conf/spark-defaults.conf <<EOL
spark.history.fs.logDirectory=s3a://$S3_BUCKET_NAME/spark-logs
spark.hadoop.fs.s3a.endpoint=$S3_ENDPOINT
spark.hadoop.fs.s3a.access.key=$S3_ACCESS_KEY
spark.hadoop.fs.s3a.secret.key=$S3_SECRET_KEY
spark.hadoop.fs.s3a.impl=org.apache.hadoop.fs.s3a.S3AFileSystem
spark.hadoop.fs.s3a.path.style.access=true
EOL
