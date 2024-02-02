import os
import requests

os.environ["SOLR_USERNAME"] = "admin"
os.environ["SOLR_PASSWORD"] = "3gzOwNhko"
os.environ["SOLR_URL"] = "solr.default.svc.cluster.local:8983"

def check_solr_status():
    username = os.environ['SOLR_USERNAME']
    password = os.environ['SOLR_PASSWORD']
    solr_url = os.environ['SOLR_URL']

    # Check if the URL is well-formed
    if not solr_url.startswith("http://") and not solr_url.startswith("https://"):
        solr_url = f"http://{solr_url}/solr"

    # Append the Solr API admin path
    solr_url = f"{solr_url}"

    try:
        response = requests.get(solr_url, auth=(username, password))
        print(response.text)
        response.raise_for_status()

        json_response = response.json()
        solr_status = json_response['status']

        if solr_status == 0:
            print("Solr is running and operational.")
        else:
            print("Solr is running but not operational. Status code:", solr_status)

    except requests.exceptions.HTTPError as e:
        print(f"An HTTP error occurred: {e}")
    except requests.exceptions.RequestException as e:
        print(f"A request error occurred: {e}")

if __name__ == "__main__":
    
    check_solr_status()
