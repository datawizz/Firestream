import os
import unittest
import boto3
from botocore.exceptions import NoCredentialsError, ClientError

class TestS3Access(unittest.TestCase):

    def setUp(self):
        self.s3 = boto3.client(
            's3',
            aws_access_key_id=os.environ['S3_LOCAL_ACCESS_KEY_ID'],
            aws_secret_access_key=os.environ['S3_LOCAL_SECRET_ACCESS_KEY'],
            # region_name=os.environ['S3_LOCAL_DEFAULT_REGION'],
            endpoint_url=f"http://{os.environ['S3_LOCAL_ENDPOINT_URL']}"
        )
        self.bucket_name = os.environ['S3_LOCAL_BUCKET_NAME']

    def test_list_buckets(self):
        try:
            response = self.s3.list_buckets()
            self.assertIn('Buckets', response)
        except (NoCredentialsError, ClientError) as e:
            self.fail(f"Failed to list buckets: {e}")

    def test_bucket_exists(self):
        try:
            self.s3.head_bucket(Bucket=self.bucket_name)
        except (NoCredentialsError, ClientError) as e:
            self.fail(f"Failed to access the bucket '{self.bucket_name}': {e}")

    def test_bucket_operations(self):
        test_key = 'test_key.txt'
        test_data = b'This is test data.'

        # Test uploading an object to the bucket
        try:
            self.s3.put_object(Bucket=self.bucket_name, Key=test_key, Body=test_data)
        except (NoCredentialsError, ClientError) as e:
            self.fail(f"Failed to upload the object '{test_key}': {e}")

        # Test downloading the object from the bucket
        try:
            response = self.s3.get_object(Bucket=self.bucket_name, Key=test_key)
            downloaded_data = response['Body'].read()
            self.assertEqual(test_data, downloaded_data)
        except (NoCredentialsError, ClientError) as e:
            self.fail(f"Failed to download the object '{test_key}': {e}")

        # Test deleting the object from the bucket
        try:
            self.s3.delete_object(Bucket=self.bucket_name, Key=test_key)
        except (NoCredentialsError, ClientError) as e:
            self.fail(f"Failed to delete the object '{test_key}': {e}")



def test_s3_etl_lib():

    from etl_lib import DataContext, DataModel, DataSink
    




if __name__ == '__main__':
    unittest.main()
