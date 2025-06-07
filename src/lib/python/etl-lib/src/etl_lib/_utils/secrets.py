import json
import os
from google.cloud import secretmanager
import tempfile


class Secrets:
    """
    Provides access to secrets as a JSON object
    Uses Google Cloud Platform Secret Manager as a backend
    Designed to be used only at runtime
    """

    def __init__(self) -> None:
        """
        Pull the Google Application Credentials Certificate from various sources
        setup the persistant clinet object
        """
        try:
            self.client = self.default_credentials()
        except:
            self.client = self.get_from_env()

    def default_credentials(self):
        """
        Assume that application default credentials have already been set.


        Assume that gcloud auth application-default login has been run and the user
        has logged in via the browser.
        """
        client = secretmanager.SecretManagerServiceClient()
        # In development this is set using the .devcontainer/.env
        self.project_id = os.environ.get("GCP_PROJECT_ID")
        return client

    def get_from_env(self):
        """
        Creates a client using credentials from environment variables.
        """

        # The default application credentials should always be specified (even if not populated)
        default_credentials_path = os.environ.get("GOOGLE_APPLICATION_CREDENTIALS_PATH")

        assert default_credentials_path is not None

        if not os.path.exists(default_credentials_path):
            _file = tempfile.NamedTemporaryFile()
            _file = _file.name
        else:
            _file = default_credentials_path

        # If the raw certificate is available then load it
        raw_cert = os.environ.get("GCP_CRED_RAW")

        CERT_OBJ = json.loads(raw_cert)

        self.project_id = CERT_OBJ.get("project_id")

        with open(_file, "w") as f:
            f.write(json.dumps(CERT_OBJ))

        # Ensure the cert is usable
        client = secretmanager.SecretManagerServiceClient.from_service_account_file(
            filename=_file
        )
        return client

    def get_secret(self, secret_id: str):

        """
        Query the GCP secrets endpoint by key
        Assume that secrets are created in advance and populated with a offline process
        "Secrets" retrived in this way are injected at run time to containers and never uploaded to another service
        """

        # Build the resource name of the secret.
        name = f"projects/{self.project_id}/secrets/{secret_id}/versions/latest"

        # Access the secret version.
        response = self.client.access_secret_version(name=name)

        payload = response.payload.data.decode("utf-8")

        return json.loads(payload)
