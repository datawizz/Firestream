# Use the dependencies image
ARG DEPENDENCIES
FROM $DEPENDENCIES as dependencies

# Base image for Spark
ARG SPARK_PYTHON
FROM $SPARK_PYTHON as spark

# Copy JAR files from the dependencies image
COPY --from=dependencies /usr/local/jars /usr/local/jars

# Rest of your Spark configurations, if any
