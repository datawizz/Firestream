




import pytest
import requests

class SolrClient:
    def __init__(self, solr_url):
        self.solr_url = solr_url

    def create_core(self, core_name, config_set):
        url = f"{self.solr_url}/admin/cores?action=CREATE&name={core_name}&instanceDir={core_name}&configSet={config_set}"
        response = requests.get(url)
        return response.status_code

    def search_query(self, core_name, query):
        url = f"{self.solr_url}/{core_name}/select?q={query}"
        response = requests.get(url)
        return response.status_code


def test_solr_connection():
    # Create a SolrClient object
    solr_client = SolrClient("http://solr.default.svc.cluster.local:8983")

    # Try to create a new core
    status_code = solr_client.create_core("sample_techproducts", "sample_techproducts_configs")

    # Assert that the status code is 200
    assert status_code == 200, "Unable to create core, connection may have failed."

    # Try to perform a search query
    status_code = solr_client.search_query("sample_techproducts", "*:*")

    # Assert that the status code is 200
    assert status_code == 200, "Unable to execute search query, connection may have failed."


if __name__ == "__main__":
    test_solr_connection()
