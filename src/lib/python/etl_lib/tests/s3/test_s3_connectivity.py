#!/usr/bin/env python3

"""
This test script measures the time it takes to upload and download a 10 MB object using the S3 API.
"""

import boto3
import time
import os


def _client():

    # Specify your AWS credentials and the name of the S3 bucket you want to test
    bucket_name = os.environ['S3_LOCAL_BUCKET_NAME']

    # Create an S3 client
    s3_client = boto3.client(
                's3',
                aws_access_key_id=os.environ['S3_LOCAL_ACCESS_KEY_ID'],
                aws_secret_access_key=os.environ['S3_LOCAL_SECRET_ACCESS_KEY'],
                region_name=os.environ['S3_LOCAL_DEFAULT_REGION'],
                endpoint_url=os.environ['S3_LOCAL_ENDPOINT_URL']
            )
    return s3_client, bucket_name

def test_connection():

    s3_client, bucket_name = _client()

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
    import pytest
    pytest.main([__file__])