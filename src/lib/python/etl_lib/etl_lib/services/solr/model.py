

import os
from xml.etree.ElementTree import Element, SubElement, tostring, indent
from pydantic import BaseModel
from typing import List, Union

class SolrField(BaseModel):
    name: str
    type: str
    indexed: bool
    stored: bool
    required: bool = False
    multiValued: bool = False
    default: Union[str, None] = None

class SolrSchema(BaseModel):
    name: str
    version: str
    uniqueKey: str
    fields: List[SolrField]


class SolrClient:
    """
    SolrClient is a wrapper around the Solr API.
    """

    def __init__(self):
        """
        Initialize the SolrClient.
        Load environment variables.
        """
        self.solr_url = os.environ["SOLR_URL"]
        self.solr_port = os.environ["SOLR_PORT"]
        self.solr_username = os.environ["SOLR_USERNAME"]
        self.solr_password = os.environ["SOLR_PASSWORD"]
        self.solr_default_core = os.environ["SOLR_DEFAULT_CORE"]



    def generate_solr_schema_xml(self, schema: SolrSchema) -> str:
        root = Element("schema")
        root.set("name", schema.name)
        root.set("version", schema.version)

        for field in schema.fields:
            field_element = SubElement(root, "field")
            field_element.set("name", field.name)
            field_element.set("type", field.type)
            field_element.set("indexed", str(field.indexed).lower())
            field_element.set("stored", str(field.stored).lower())
            field_element.set("required", str(field.required).lower())
            field_element.set("multiValued", str(field.multiValued).lower())

            if field.default:
                field_element.set("default", field.default)

        unique_key = SubElement(root, "uniqueKey")
        unique_key.text = schema.uniqueKey

        indent(root, space="  ")

        return tostring(root, encoding="UTF-8", method="xml").decode("UTF-8")




    def generate_configset(self, schema: SolrSchema, configset_dir: str):


        _schema = self.generate_solr_schema_xml(schema)
        print(_schema)

        # Save the generated schema.xml file
        os.makedirs(configset_dir, exist_ok=True)
        with open(os.path.join(configset_dir, "schema.xml"), "w") as f:
            f.write(_schema)

        # # Copy the default solrconfig.xml and other config files from the _default configSet
        # default_configset_dir = "<solr_install_dir>/server/solr/configsets/_default/conf"
        # for file in os.listdir(default_configset_dir):
        #     if file != "schema.xml":
        #         shutil.copy(os.path.join(default_configset_dir, file), os.path.join(configset_dir, file))




if __name__ == "__main__":

    c = SolrClient()

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
    configset_dir = "src/lib/python/etl_lib/etl_lib/services/solr/config"
    c.generate_configset(schema, configset_dir)
