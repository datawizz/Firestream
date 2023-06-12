ARG BASE_IMAGE
ARG SPARK_HOME
FROM ${BASE_IMAGE} as builder



# Use an official OpenJDK image as the base image
FROM openjdk:8-jdk-slim as runtime


# Set the environment variables
ENV PATH=$PATH:$SPARK_HOME/bin
ENV SPARK_HOME=$SPARK_HOME

# Copy the built Spark to the image
COPY --from=builder $SPARK_HOME $SPARK_HOME

# Expose port 18080 for the History Server
EXPOSE 18080

# Copy the configure-spark.sh script
COPY configure_spark.sh $SPARK_HOME/sbin/configure_spark.sh


# Run the configure-spark.sh script
RUN $SPARK_HOME/sbin/configure_spark.sh

# Start the Spark History Server
CMD ["bash", "/spark/sbin/start-history-server.sh"]
