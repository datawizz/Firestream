
# TODO SSL on everything!
import warnings
from urllib3.exceptions import InsecureRequestWarning
warnings.simplefilter('ignore', InsecureRequestWarning)



import os
import hashlib
from opensearchpy import OpenSearch, exceptions

class OpenSearchClient:
    def __init__(self):
        self.client = OpenSearch(
            hosts=[os.environ['OPENSEARCH_URL']],
            http_auth=(os.environ['OPENSEARCH_USERNAME'], os.environ['OPENSEARCH_PASSWORD']),
            verify_certs=False,
        )

    def write(self, index, data):
        doc_id = hashlib.md5(data['prolog_line'].encode()).hexdigest()
        return self.client.index(index=index, id=doc_id, body=data)

    def read(self, index, query):
        return self.client.search(index=index, body=query)

def read_prolog_file(filename):
    with open(filename, 'r') as file:
        for line in file:
            yield line.strip()

if __name__ == '__main__':
    _PATH = "/workspace/src/lib/python/etl_lib/tests/example_data/simpsons.pl"
    
    client = OpenSearchClient()
    
    for line in read_prolog_file(_PATH):
        data = {'prolog_line': line}
        write_response = client.write('prolog_data', data)
        print(write_response)
    
    simpsons_family = ["homer", "marge", "bart", "lisa", "maggie", "abe", "mona"]
    
    for member in simpsons_family:
        query = {
            "query": {
                "match": {
                    "prolog_line": member
                }
            }
        }
        
        search_response = client.read('prolog_data', query)
        print(f"Results for {member}:")
        
        if search_response.get("hits", {}).get("total", {}).get("value", 0) > 0:
            for hit in search_response["hits"]["hits"]:
                print(hit["_source"]["prolog_line"])
        else:
            print(f"No matches found for {member}")
        print("\n")
