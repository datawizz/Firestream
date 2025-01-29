import unittest
import os
from etl_lib.services.solr.client import SolrClient

class TestSolrClient(unittest.TestCase):

    def setUp(self):
        self.solr_url = os.environ.get('SOLR_URL') or "https://127.0.0.1:8983"
        self.core_name = "my-core"
        self.solr_client = SolrClient(self.solr_url, self.core_name)

    def test_create_read_update_delete_document(self):
        print("Testing Create Document")
        document = {
            "id": "1",
            "title": "Test Title",
            "content": "Test content for the document",
            "timestamp": "2023-04-21T00:00:00Z"
        }
        self.solr_client.create_document(document)

        print("Testing Read Document")
        retrieved_document = self.solr_client.read_document("1")
        self.assertIsNotNone(retrieved_document)
        self.assertEqual(retrieved_document["title"], "Test Title")

        print("Testing Update Document")
        updated_fields = {
            "title": "Updated Test Title"
        }
        self.solr_client.update_document("1", updated_fields)
        updated_document = self.solr_client.read_document("1")
        self.assertEqual(updated_document["title"], "Updated Test Title")

        print("Testing Delete Document")
        self.solr_client.delete_document("1")
        deleted_document = self.solr_client.read_document("1")
        self.assertIsNone(deleted_document)

    def test_vector_search(self):
        print("Testing Vector Search")
        document = {
            "id": "2",
            "title": "Another Test Title",
            "content": "Some content for another document",
            "timestamp": "2023-04-22T00:00:00Z"
        }
        self.solr_client.create_document(document)

        search_results = self.solr_client.vector_search("title:Another*")
        self.assertTrue(len(search_results) > 0)
        self.solr_client.delete_document("2")

if __name__ == "__main__":
    unittest.main(verbosity=2)
