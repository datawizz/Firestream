from confluent_kafka import Consumer, Producer, KafkaError
import json
import os

class KafkaJsonHandler:
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

    def _append_to_file(self, data_chunk, file_name):
        with open(file_name, 'a') as f:
            for item in data_chunk:
                json.dump(item, f)
                f.write('\n')

    def _read_messages_from_json(self, file_name):
        with open(file_name, 'r') as f:
            for line in f:
                yield json.loads(line.strip())

    def read_from_topic_write_to_json(self, file_name):
        consumer = Consumer(self.consumer_conf)
        consumer.subscribe([self.topic_name])

        data_chunk = []

        while True:
            msg = consumer.poll(1.0)
            if msg is None:
                continue
            if msg.error():
                if msg.error().code() == KafkaError._PARTITION_EOF:
                    if data_chunk:
                        self._append_to_file(data_chunk, file_name)
                    break
                else:
                    print('Error while consuming message: {}'.format(msg.error()))
            else:
                data_chunk.append(json.loads(msg.value().decode('utf-8')))
                if len(data_chunk) == self.chunk_size:
                    self._append_to_file(data_chunk, file_name)
                    data_chunk.clear()

        consumer.close()

    def read_from_json_write_to_topic(self, file_name):
        producer = Producer(self.producer_conf)
        messages = self._read_messages_from_json(file_name)

        for message in messages:
            producer.produce(self.topic_name, json.dumps(message))

        producer.flush()

if __name__ == "__main__":
    handler = KafkaJsonHandler(broker_address=os.environ['KAFKA_BOOTSTRAP_SERVER'], topic_name='A_RUSTY_TOPIC', group_id='group_id')
    handler.read_from_topic_write_to_json('output_file.ndjson')
    # handler.read_from_json_write_to_topic('output_file.ndjson')
