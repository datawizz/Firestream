

# This script serves to push high volumes of data through the system
# It is intended to max the CPU of the host then collect latency metrics
# When operating streaming applications it is important to understand the latency of the system

# schema
# timestamp_created - the local host time when the record was created in the spark application
# kafka_write_1 - The timestamp recorded by Kafka when the record is first created
# timestamp_read_1 - the timestamp that the record was read by the spark application
# kafka_write_2 - the timestamp that the record was written to the output topic
# timestamp_read_2 - the timestamp that the record was read from the output topic

# The goal is to understand the latency between the timestamps

# timestamp_created - timestamp_read = the latency of the spark application as measured by when it would be able to compute on the record


# 1. Start the Spark Metronome job