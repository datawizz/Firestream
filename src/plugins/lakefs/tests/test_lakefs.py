

# # TODO this hack is to set the LakeFS credentials, which should be set via the CLI (but cannot be used with MinIO :|  )
# LAKEFS_ACCESS_KEY = "AKIAJV77CIW6QKTEQWSQ"
# LAKEFS_SECRET_KEY = "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"
# LAKEFS_ENDPOINT = "http://lakefs.default.svc.cluster.local"
# REPOSITORY = "fireworks_test_repo"


# # import os
# # LAKEFS_ACCESS_KEY = os.environ["LAKEFS_ACCESS_KEY"]
# # LAKEFS_SECRET_KEY = os.environ["LAKEFS_SECRET_KEY"]
# # LAKEFS_ENDPOINT = os.environ["LAKEFS_ENDPOINT"]


# import random
# import string
# from datetime import datetime
# import lakefs_client
# from lakefs_client.api import branches_api, commits_api
# from lakefs_client.model.branch_creation import BranchCreation
# from lakefs_client.model.commit_creation import CommitCreation
# from pyspark.sql import SparkSession



# charset = string.ascii_lowercase
# now_datetime = datetime.now()

# def get_random_string(length):
#     return "".join(random.choice(charset) for i in range(length))

# def create_sample_data(num_rows):
#     data = []
#     for i in range(num_rows):
#         d = {
#             "yyyy_mm_dd": now_datetime.strftime("%Y-%m-%d"),
#             "hh_mm": now_datetime.strftime("%H-%M"),
#             "i": i,
#             "datetime_now": now_datetime,
#             "random_text": get_random_string(5),
#             "random_float": round(random.uniform(0.01, 10000.01), 2),
#         }
#         data.append(d)
#     return data

# def create_sample_df():
#     data = create_sample_data(100)
#     spark = SparkSession.builder.getOrCreate()
#     df = spark.createDataFrame(data)
#     df.createOrReplaceTempView("df")
#     return df

# def configure_lakefs():
#     spark = SparkSession.builder.getOrCreate()
#     spark.sparkContext._jsc.hadoopConfiguration().set("fs.lakefs.access.key", LAKEFS_ACCESS_KEY)
#     spark.sparkContext._jsc.hadoopConfiguration().set("fs.lakefs.secret.key", LAKEFS_SECRET_KEY)
#     spark.sparkContext._jsc.hadoopConfiguration().set("fs.lakefs.endpoint", LAKEFS_ENDPOINT)

# spark = SparkSession.builder.getOrCreate()

# configuration = lakefs_client.Configuration(
#     host=LAKEFS_ENDPOINT,
#     username=LAKEFS_ACCESS_KEY,
#     password=LAKEFS_SECRET_KEY,
# )

# now_datetime = datetime.now()
# now_yyyy_mm_dd = now_datetime.strftime("%Y-%m-%d")
# now_hh_mm = now_datetime.strftime("%H-%M")
# branch_name = f"branch_{now_yyyy_mm_dd}_{now_hh_mm}"

# with lakefs_client.ApiClient(configuration) as api_client:
#     api_instance = branches_api.BranchesApi(api_client)
#     repository = "your_repo_name_here"
#     branch_creation = BranchCreation(name=branch_name, source="main")
#     try:
#         api_instance.create_branch(repository, branch_creation)
#     except lakefs_client.ApiException as e:
#         raise

# df = create_sample_df()
# configure_lakefs()

# output = f"lakefs://your_repo_name_here/{branch_name}/data/sample_data_lakefs/"
# df.write.partitionBy(["yyyy_mm_dd", "hh_mm"]).parquet(output)

# with lakefs_client.ApiClient(configuration) as api_client:
#     api_instance = commits_api.CommitsApi(api_client)

#     commit_creation = CommitCreation(message="example commit message")
#     try:
#         api_instance.commit(REPOSITORY, branch_name, commit_creation)
#     except lakefs_client.ApiException as e:
#         raise
