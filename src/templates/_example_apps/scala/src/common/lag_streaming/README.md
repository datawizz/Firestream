# Lag Streaming

Spark when run in Batch mode, or streaming with a "Complete" output mode, offers a function named Lag. Lag finds the N-M record in a stream of N records where M is the offset in the ordered list of records. There is no such function available for a streaming context as the ordering cannot be guaranteed (normally). Records from Kafka do have a gaurentee by key if configured correctly.
