
import os

HOST = os.environ['OPENSEARCH_URL']
USERNAME = os.environ['OPENSEARCH_USERNAME']
PASSWORD = os.environ['OPENSEARCH_PASSWORD']
HOST = "https://opensearch-cluster-master.default.svc.cluster.local:9200"
USERNAME = "admin"
PASSWORD = "admin"


command = "curl -XGET https://opensearch-cluster-master.default.svc.cluster.local:9200 -u 'admin:admin' --insecure"



from opensearchpy import OpenSearch, helpers
import faker

# Create a connection
os = OpenSearch(
    [HOST],
    http_auth=(USERNAME, PASSWORD),
    use_ssl=True,
    verify_certs=False
)

# Faker library for data generation
fake = faker.Faker()

# Define a function that creates fake documents
def create_fake_docs(index, num=100):
    for _ in range(num):
        yield {
            "_index": index,
            "_source": {
                "name": fake.name(),
                "email": fake.email(),
                "birthdate": fake.date_of_birth().isoformat(),
            }
        }

# Define the index name
index_name = 'test-index'

# Create the index
os.indices.create(index=index_name, ignore=400)

# Use the helpers library's bulk function to insert the documents
helpers.bulk(os, create_fake_docs(index_name))

# Refresh the index to make sure all documents have been processed and are searchable
os.indices.refresh(index=index_name)

# Now let's perform a search on the index
res = os.search(index=index_name, body={"query": {"match_all": {'_name': 'Ca'}}})

print(res)

# Print out the result
for hit in res['hits']['hits']:
    print(f"Document ID: {hit['_id']}, Document: {hit['_source']}")
