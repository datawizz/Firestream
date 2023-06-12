from pyspark.sql import SparkSession
from pyspark.sql.functions import col

spark = SparkSession.builder \
    .appName("LowLatencyKafkaTransactionalStreaming") \
    .getOrCreate()

# Kafka consumer configuration
kafka_consumer_params = {
    "kafka.bootstrap.servers": "localhost:9092",
    "subscribe": "input_topic",
    "startingOffsets": "latest",
    "isolation.level": "read_committed",
    "group.id": "kafka_transactional_streaming_group",
    "max.poll.interval.ms": "300000",
    "max.poll.records": "500",
    "fetch.min.bytes": "1"
}

# Read the input stream from Kafka
input_stream = spark.readStream \
    .format("kafka") \
    .options(**kafka_consumer_params) \
    .load()

# Process the input stream (perform transformations, aggregations, etc.)
processed_stream = input_stream.selectExpr("CAST(key AS STRING)", "CAST(value AS STRING)")

# Kafka producer configuration
kafka_producer_params = {
    "kafka.bootstrap.servers": "localhost:9092",
    "topic": "output_topic",
    "checkpointLocation": "/path/to/checkpoint/dir",
    "kafka.enable.idempotence": "true",
    "kafka.max.inflight.requests.per.connection": "1",
    "kafka.acks": "all",
    "kafka.linger.ms": "1",
    "kafka.batch.size": "16384",
    "kafka.request.timeout.ms": "30000"
}

# Write the output stream to Kafka
processed_stream.writeStream \
    .format("kafka") \
    .options(**kafka_producer_params) \
    .trigger(processingTime="100 milliseconds") \
    .start() \
    .awaitTermination()
