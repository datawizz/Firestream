import os
import requests



# Read environment variables
solr_url = os.environ['SOLR_URL']
solr_port = os.environ['SOLR_PORT']
solr_username = os.environ['SOLR_USERNAME']
solr_password = os.environ['SOLR_PASSWORD']
solr_default_core = os.environ['SOLR_DEFAULT_CORE']


endpoints = [
    "/solr/admin/collections?action=LIST",
    "/solr/admin/configs?action=LIST&omitHeader=true",
    "/admin/collections?action=REINDEXCOLLECTION&name=name",
    "/api/collections/my-collection/config"
]


# Construct Solr connection URL
solr_connection_url = f"{solr_url}/api/collections/my-collection/config"

def check_solr_connectivity(url, username, password):
    try:
        response = requests.get(url, auth=(username, password))

        print(response.status_code)
        print(response.text)
        if response.status_code == 200:
            print("Successfully connected to Solr cluster.")
        else:
            print(f"Error connecting to Solr cluster. Status code: {response.status_code}")
    except requests.exceptions.RequestException as e:
        print(f"Error connecting to Solr cluster: {e}")

# Check connectivity
check_solr_connectivity(solr_connection_url, solr_username, solr_password)
