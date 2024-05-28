from confluent_kafka import Consumer, Producer, KafkaError
import polars as pl
import os
import json

class KafkaParquetHandler:
    def __init__(self, broker_address, topic_name, group_id, chunk_size=10000):
        self.broker_address = broker_address
        self.topic_name = topic_name
        self.chunk_size = chunk_size
        self.consumer_conf = {
            'bootstrap.servers': self.broker_address,
            'group.id': group_id,
            'auto.offset.reset': 'earliest'
        }
        self.producer_conf = {
            'bootstrap.servers': self.broker_address,
        }

    def _append_to_parquet(self, data_chunk, file_name):
        if os.path.exists(file_name):
            df_existing = pl.read_parquet(file_name)
            df_new_chunk = pl.DataFrame(data_chunk)
            df_combined = df_existing.vstack(df_new_chunk)
            df_combined.write_parquet(file_name)
        else:
            pl.DataFrame(data_chunk).write_parquet(file_name)

    def _read_messages_from_parquet(self, file_name):
        return pl.read_parquet(file_name).to_dict(orient="records")

    def read_from_topic_write_to_parquet(self, file_name):
        consumer = Consumer(self.consumer_conf)
        consumer.subscribe([self.topic_name])

        data_chunk = []
        chunk_count = 0

        while True:
            msg = consumer.poll(1.0)
            if msg is None:
                continue
            if msg.error():
                if msg.error().code() == KafkaError._PARTITION_EOF:
                    if data_chunk:
                        self._append_to_parquet(data_chunk, file_name)
                    break
                else:
                    print('Error while consuming message: {}'.format(msg.error()))
            else:
                payload = {
                    'timestamp': msg.timestamp()[1],
                    'key': msg.key(),
                    'value': msg.value()
                }
                data_chunk.append(payload)
                if len(data_chunk) == self.chunk_size:
                    self._append_to_parquet(data_chunk, file_name)
                    chunk_count += 1
                    print(f"Writing chunk {chunk_count}")
                    data_chunk.clear()

        consumer.close()

    def read_from_parquet_write_to_topic(self, file_name):
        producer = Producer(self.producer_conf)
        messages = self._read_messages_from_parquet(file_name)

        for message in messages:
            producer.produce(self.topic_name, key=message['key'], value=message['value'])

        producer.flush()

if __name__ == "__main__":
    handler = KafkaParquetHandler(broker_address=os.environ['KAFKA_BOOTSTRAP_SERVER'], topic_name='A_RUSTY_TOPIC', group_id='group_id2')
    handler.read_from_topic_write_to_parquet('output_file.parquet')
    # handler.read_from_parquet_write_to_topic('output_file.parquet')
