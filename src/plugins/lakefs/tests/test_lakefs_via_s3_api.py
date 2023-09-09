#!/usr/bin/env python3

"""
This test script measures the time it takes to upload and download a 10 MB object using the S3 API.
"""

import boto3
import time
import os

os.environ['LAKEFS_BUCKET_NAME'] = 'fireworks'


def _client():

    # Create an S3 client
    s3_client = boto3.client(
                's3',
                aws_access_key_id=os.environ['LAKEFS_ACCESS_KEY'],
                aws_secret_access_key=os.environ['LAKEFS_SECRET_KEY'],
                region_name=os.environ['LAKEFS_DEFAULT_REGION'],
                endpoint_url=os.environ['LAKEFS_ENDPOINT_URL'],
                verify=False
            )
    return s3_client


def create_bucket(CLIENT):
    bucket_name = os.environ['LAKEFS_BUCKET_NAME']
    response = CLIENT.create_bucket(Bucket=bucket_name)
    return response


def test_connection():

    s3_client= _client()

    # Create a bucket
    # create_bucket(s3_client)

    bucket_name = os.environ['LAKEFS_BUCKET_NAME']

    # Create a test object and upload it to the bucket
    object_key = 'test-object'
    object_data = b'0' * 1024 * 1024 * 1000  # 10 MB of data
    s3_client.put_object(Bucket=bucket_name, Key=object_key, Body=object_data)

    # Measure the time it takes to download the object
    start_time = time.monotonic()
    s3_client.download_file(bucket_name, object_key, '/dev/null')
    end_time = time.monotonic()
    download_time = end_time - start_time
    print(f'Download time: {download_time:.3f} seconds')

    # Measure the time it takes to upload the object
    start_time = time.monotonic()
    s3_client.upload_file('/dev/zero', bucket_name, object_key)
    end_time = time.monotonic()
    upload_time = end_time - start_time
    print(f'Upload time: {upload_time:.3f} seconds')

    # Delete the test object
    s3_client.delete_object(Bucket=bucket_name, Key=object_key)


if __name__ == '__main__':
    test_connection()