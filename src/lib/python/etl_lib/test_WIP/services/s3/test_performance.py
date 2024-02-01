import boto3
import time
import os

# Specify your AWS credentials and the name of the S3 bucket you want to test
bucket_name = os.environ['S3_LOCAL_BUCKET_NAME']

# Create an S3 client
s3 = boto3.client(
            's3',
            aws_access_key_id=os.environ['S3_LOCAL_ACCESS_KEY_ID'],
            aws_secret_access_key=os.environ['S3_LOCAL_SECRET_ACCESS_KEY'],
            # region_name=os.environ['S3_LOCAL_DEFAULT_REGION'],
            endpoint_url=f"http://{os.environ['S3_LOCAL_ENDPOINT_URL']}"
        )

# Create a test object and upload it to the bucket
object_key = 'test-object'
object_data = b'0' * 1024 * 1024 * 1000  # 10 MB of data
s3.put_object(Bucket=bucket_name, Key=object_key, Body=object_data)

# Measure the time it takes to download the object
start_time = time.monotonic()
s3.download_file(bucket_name, object_key, '/dev/null')
end_time = time.monotonic()
download_time = end_time - start_time
print(f'Download time: {download_time:.3f} seconds')

# Measure the time it takes to upload the object
start_time = time.monotonic()
s3.upload_file('/dev/zero', bucket_name, object_key)
end_time = time.monotonic()
upload_time = end_time - start_time
print(f'Upload time: {upload_time:.3f} seconds')

# Delete the test object
s3.delete_object(Bucket=bucket_name, Key=object_key)
