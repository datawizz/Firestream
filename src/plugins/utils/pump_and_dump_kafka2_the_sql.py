from confluent_kafka import Consumer, Producer, KafkaError
import polars as pl
import os
import time
from concurrent.futures import ThreadPoolExecutor

class KafkaParquetHandler:
    def __init__(self, broker_address, topic_name, group_id, chunk_size=10000):
        self.broker_address = broker_address
        self.topic_name = topic_name
        self.chunk_size = chunk_size
        self.consumer_conf = {
            'bootstrap.servers': self.broker_address,
            'group.id': group_id,
            'auto.offset.reset': 'earliest',
            'fetch.max.bytes': 1024*1024*50,  # 50 MB
        }
        self.producer_conf = {
            'bootstrap.servers': self.broker_address,
        }

        self.path = "/workspace/src/lib/python/etl_lib/utils"
        self.filename = F"{self.path}/output_file"


    def _write_chunk_to_parquet(self, data_chunk):
        timestamp = int(time.time())
        file_name = f"{self.filename}_chunk_{timestamp}.parquet"
        df = pl.DataFrame(data_chunk)
        df.write_parquet(file_name, use_pyarrow=True)
        return file_name

    def _read_messages_from_parquet(self):
        return pl.read_parquet(self.filename).to_dict(orient="records")

    def read_from_topic_write_to_parquet(self):
        consumer = Consumer(self.consumer_conf)
        consumer.subscribe([self.topic_name])
        data_chunk = []
        chunk_count = 0

        with ThreadPoolExecutor() as executor:
            while True:
                msg = consumer.poll(1.0)
                if msg is None:
                    continue
                if msg.error():
                    if msg.error().code() == KafkaError._PARTITION_EOF:
                        if data_chunk:
                            executor.submit(self._write_chunk_to_parquet, data_chunk)
                        break
                    else:
                        print(f'Error while consuming message: {msg.error()}')
                else:
                    payload = {
                        'timestamp': msg.timestamp()[1],
                        'key': msg.key(),
                        'value': msg.value()
                    }
                    data_chunk.append(payload)
                    if len(data_chunk) == self.chunk_size:
                        executor.submit(self._write_chunk_to_parquet, data_chunk)
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
    handler = KafkaParquetHandler(broker_address=os.environ['KAFKA_BOOTSTRAP_SERVER'], topic_name='A_RUSTY_TOPIC', group_id='group_id')
    handler.read_from_topic_write_to_parquet()
