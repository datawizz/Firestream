from confluent_kafka import Consumer, Producer, KafkaError
import polars as pl
import os
import json
import boto3
import io



S3_LOCAL_BUCKET_NAME='rustybull'
S3_LOCAL_ACCESS_KEY_ID='dWAPcElxxo8TWhHorffi'
S3_LOCAL_SECRET_ACCESS_KEY='QOz9AR1sVKaRGQAJg1ZecNQvBueAKHuTFzWSPucF'
S3_LOCAL_ENDPOINT_URL='http://10.0.0.69:9000'



class KafkaParquetHandler:

    def __init__(self, broker_address, topic_name, group_id, chunk_size=100000):
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


    def _upload_to_s3(self, data_chunk, s3_key):

        self.s3_bucket = S3_LOCAL_BUCKET_NAME

        self.s3_client = boto3.client(
                's3',
                aws_access_key_id=S3_LOCAL_ACCESS_KEY_ID,
                aws_secret_access_key=S3_LOCAL_SECRET_ACCESS_KEY,
                region_name='us-east-1',
                endpoint_url=S3_LOCAL_ENDPOINT_URL
        )
        with io.BytesIO() as buffer:
            pl.DataFrame(data_chunk).write_parquet(buffer)
            buffer.seek(0)
            self.s3_client.upload_fileobj(buffer, self.s3_bucket, s3_key)

    def read_from_topic_write_to_parquet(self, base_file_name):
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
                        s3_key = f"{base_file_name}_chunk_{chunk_count}.parquet"
                        self._upload_to_s3(data_chunk, s3_key)
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
                    s3_key = f"{base_file_name}_chunk_{chunk_count}.parquet"
                    self._upload_to_s3(data_chunk, s3_key)
                    chunk_count += 1
                    print(f"Uploaded chunk {chunk_count} to S3")
                    data_chunk.clear()

        consumer.close()

if __name__ == "__main__":

    handler = KafkaParquetHandler(broker_address=os.environ['KAFKA_BOOTSTRAP_SERVER'], topic_name='A_RUSTY_TOPIC', group_id='group_id2')
    handler.read_from_topic_write_to_parquet('output_file')
