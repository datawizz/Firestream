# Exactly Once

Kafka provides exactly once guarantees under certain conditions. This example demonstrates how to use the exactly once feature of Kafka.


1. Ingest a local csv file into Kafka using Spark. This ensures that the data is written to Kafka exactly once. The data is written to a topic called `exactly-once-input`.
2. Use several spark applications to process the data
3. Observe the output and ensure it matches the input