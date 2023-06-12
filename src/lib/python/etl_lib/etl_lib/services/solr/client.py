#python
import requests
import json
import os
from xmlbuilder import XMLBuilder
from pydantic import BaseModel
from typing import List, Union

from etl_lib.model import DataModel


class SolrField(DataModel):
    """
    A Solr field includes the information required to index it
    #TODO what else is required for different types of indexing?
    """
    name: str
    type: str
    indexed: bool
    stored: bool
    required: bool = False
    multiValued: bool = False
    default: Union[str, None] = None

class SolrSchema(DataModel):
    name: str
    version: str
    uniqueKey: str
    fields: List[SolrField]




class SolrClient:
    """
    The solr client interacts with the Solr Cloud service.

    The client is responsible for:
        - Checking the status of the Solr Cloud service
        - Creating a new Solr core
        - Managing the schema of a Solr core including evolution of the schema
        - Deleting an existing Solr core
        - Indexing documents into a Solr core
        - Querying documents from a Solr core

    <fieldType name="knn_vector" class="solr.DenseVectorField" vectorDimension="4" similarityFunction="cosine"/>
    <field name="vector" type="knn_vector" indexed="true" stored="true"/>
    https://solr.apache.org/guide/solr/latest/query-guide/dense-vector-search.html

    """



    def __init__(self):
        self.solr_url = os.environ['SOLR_URL']
        # self.core_name = core_name
        # self.base_url = f"{solr_url}/{core_name}"




    def generate_solr_schema_xml(schema: SolrSchema) -> str:
        with XMLBuilder("schema", encoding="UTF-8") as x:
            x._attr("name", schema.name)
            x._attr("version", schema.version)

            for field in schema.fields:
                with x.field():
                    x._attr("name", field.name)
                    x._attr("type", field.type)
                    x._attr("indexed", str(field.indexed).lower())
                    x._attr("stored", str(field.stored).lower())
                    x._attr("required", str(field.required).lower())
                    x._attr("multiValued", str(field.multiValued).lower())

                    if field.default:
                        x._attr("default", field.default)

            x.uniqueKey(schema.uniqueKey)

        return str(x)



    def generate_configset(schema: SolrSchema, configset_dir: str):
        os.makedirs(configset_dir, exist_ok=True)

        # Save the generated schema.xml file
        with open(os.path.join(configset_dir, "schema.xml"), "w") as f:
            f.write(generate_solr_schema_xml(schema))

        # Copy the default solrconfig.xml and other config files from the _default configSet
        default_configset_dir = "<solr_install_dir>/server/solr/configsets/_default/conf"
        for file in os.listdir(default_configset_dir):
            if file != "schema.xml":
                shutil.copy(os.path.join(default_configset_dir, file), os.path.join(configset_dir, file))



    def create_document(self, document):
        url = f"{self.base_url}/update?commit=true"
        response = requests.post(url, json=document)
        response.raise_for_status()
        return response.json()

    def read_document(self, document_id):
        url = f"{self.base_url}/get?id={document_id}"
        response = requests.get(url)
        response.raise_for_status()
        return response.json()

    def update_document(self, document_id, updates):
        url = f"{self.base_url}/update?commit=true"
        atomic_update = {"id": document_id, **updates}
        response = requests.post(url, json=atomic_update)
        response.raise_for_status()
        return response.json()

    def delete_document(self, document_id):
        url = f"{self.base_url}/update?commit=true"
        delete_query = f"id:{document_id}"
        response = requests.post(url, data={"delete": delete_query})
        response.raise_for_status()
        return response.json()

    def vector_search(self, text, vector_field_name, num_results=10):
        url = f"{self.base_url}/select"
        params = {
            "q": f"{{!vector}}{vector_field_name}",
            "rq": f"{{!vector}}{vector_field_name}={text}",
            "rows": num_results,
            "fl": "*,score",
            "wt": "json",
        }
        response = requests.get(url, params=params)
        response.raise_for_status()
        return response.json()["response"]["docs"]


if __name__ == "__main__":


    SOLR_URL = "http://solr.default.svc.cluster.local"
    solr_client = SolrClient(solr_url=SOLR_URL, core_name="my-core")

    # Define the schema
    schema = SolrSchema(
        name="example",
        version="1.6",
        uniqueKey="id",
        fields=[
            SolrField(name="id", type="string", indexed=True, stored=True, required=True, multiValued=False),
            SolrField(name="title", type="text_general", indexed=True, stored=True),
            SolrField(name="content", type="text_general", indexed=True, stored=True),
            SolrField(name="timestamp", type="date", indexed=True, stored=True, default="NOW"),
        ],
    )

    # Generate the configSet and save it to a directory
    # configset_dir = "<path_to_configset_directory>"
    # generate_configset(schema, configset)



    # Create a document
    document = {"id": "1", "title": "My First Document", "content": "This is a sample document."}
    solr_client.create_document(document)

    # Read a document
    document = solr_client.read_document("1")
    print(document)

    # Update a document
    updates = {"title": {"set": "My Updated Document"}}
    solr_client.update_document("1", updates)

    # Delete a document
    solr_client.delete_document("1")

    # Vector search
    text = "sample query"
    vector_field_name = "content_vector"
    results = solr_client.vector_search(text, vector_field_name)
