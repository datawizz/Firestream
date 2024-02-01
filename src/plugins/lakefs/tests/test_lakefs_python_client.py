# import os
# os.environ["LAKEFS_ENDPOINT_URL"] = "http://lakefs.default.svc.cluster.local:80/api/v1"
# os.environ["LAKEFS_ACCESS_KEY"] = "TODO_CHANGE_ME2"
# os.environ["LAKEFS_SECRET_KEY"] = "THIS_IS_A_SECRET_TODO_CHANGE_ME2"
# BUCKET = "fireworks"
# REPO = "fireworks-test-repo"
# BRANCH = "main"
# BRANCH2 = "forked"
# CSV_DATA = "/workspace/tests/cities.csv"

# import boto3

# import lakefs_client
# from lakefs_client import models
# from lakefs_client.client import LakeFSClient





# def create_repo(client):
#     repo = models.RepositoryCreation(name=REPO, storage_namespace=f's3://{BUCKET}/repos/{REPO}', default_branch=BRANCH)
#     client.repositories.create_repository(repo)

# def reset_repo(client, boto_client):

#     # Create an S3 client
#     s3_client = boto3.client(
#                 's3',
#                 aws_access_key_id=os.environ['LAKEFS_ACCESS_KEY'],
#                 aws_secret_access_key=os.environ['LAKEFS_SECRET_KEY'],
#                 endpoint_url=os.environ['LAKEFS_ENDPOINT_URL']
#             )

#     try:
#         print(f"Deleting repo {REPO} from lakeFS")
#         client.repositories.delete_repository(REPO)
#     except Exception as e:
#         print(f"Repo {REPO} does not exist")

#     try:
#         print(f"Deleting bucket {BUCKET} from S3")
#         objs = boto_client.list_objects_v2(Bucket=BUCKET, Prefix=REPO)['Contents']
#         for obj in objs:
#             boto_client.delete_object(Bucket=BUCKET, Key=obj['Key'])

#     except Exception as e:
#         print(f"Bucket {BUCKET} does not exist")

# def list_branches(client):
#     return client.branches.list_branches(REPO).results

# def create_branch(client):
#     client.branches.create_branch(repository=REPO, branch_creation=models.BranchCreation(name='experiment-aggregations1', source=BRANCH))

# def upload_file(client, file_path):
#     with open(file_path, 'rb') as f:
#         client.objects.upload_object(repository=REPO, branch='experiment-aggregations1', path=file_path, content=f)

# def diff_branch(client):
#     return client.branches.diff_branch(repository=REPO, branch='experiment-aggregations1').results

# def commit_changes(client):
#     client.commits.commit(
#         repository=REPO,
#         branch=BRANCH2,
#         commit_creation=models.CommitCreation(message='Added a CSV file!', metadata={'using': 'python_api'})
#     )

# def diff_refs(client):
#     return client.refs.diff_refs(repository=REPO, left_ref='experiment-aggregations1', right_ref=BRANCH).results

# def merge_changes(client):
#     client.refs.merge_into_branch(repository=REPO, source_ref='experiment-aggregations1', destination_branch=BRANCH)

# if __name__ == '__main__':
#     configuration = lakefs_client.Configuration()
#     configuration.username = os.environ["LAKEFS_ACCESS_KEY"]
#     configuration.password = os.environ["LAKEFS_SECRET_KEY"]
#     configuration.host = os.environ["LAKEFS_ENDPOINT_URL"]

#     boto_client = boto_client()

#     client = LakeFSClient(configuration)
#     delete_repo(client, boto_client)

#     create_repo(client)
#     # create_branch(client)
#     # upload_file(client, CSV_DATA)
#     # commit_changes(client)
#     # merge_changes(client)
#     # delete_repo(client)



import os
import random
import string

os.environ["LAKEFS_ENDPOINT_URL"] = "http://lakefs.default.svc.cluster.local:80/api/v1"
os.environ["LAKEFS_ACCESS_KEY"] = "TODO_CHANGE_ME2"
os.environ["LAKEFS_SECRET_KEY"] = "THIS_IS_A_SECRET_TODO_CHANGE_ME2"
os.environ["LAKEFS_DEFAULT_REGION"] = "us-west-2"


REPO = "fireworks"
BUCKET = 'fireworks'
BRANCH = "main"
BRANCH2 = "forked"
CSV_DATA = "/workspace/tests/cities.csv"

import boto3
import lakefs_client
from lakefs_client import models
from lakefs_client.client import LakeFSClient




# def reset_repo(client, boto_client):
#     try:
#         client.repositories.delete_repository(REPO)
#     except Exception as e:
#         print(e)

#     try:
#         boto_client.delete_bucket(Bucket=BUCKET)
#     except boto_client.exceptions.NoSuchBucket:
#         print("Bucket does not exist, skipping deletion.")

def create_repo(client):

    repo = models.RepositoryCreation(name=REPO, storage_namespace=f's3://{BUCKET}/repos/{REPO}', default_branch=BRANCH)
    client.repositories.create_repository(repo)


if __name__ == '__main__':
    configuration = lakefs_client.Configuration()
    configuration.username = os.environ["LAKEFS_ACCESS_KEY"]
    configuration.password = os.environ["LAKEFS_SECRET_KEY"]
    configuration.host = os.environ["LAKEFS_ENDPOINT_URL"]
    configuration.verify_ssl = False


    client = LakeFSClient(configuration)
    create_repo(client)


    # boto_client = boto3.client(
    #             's3',
    #             aws_access_key_id=os.environ['LAKEFS_ACCESS_KEY'],
    #             aws_secret_access_key=os.environ['LAKEFS_SECRET_KEY'],
    #             region_name=os.environ['LAKEFS_DEFAULT_REGION'],
    #             endpoint_url=os.environ['LAKEFS_ENDPOINT_URL']
    #         )

    # create_repo(client, boto_client)
    # reset_repo(client, boto_client)
