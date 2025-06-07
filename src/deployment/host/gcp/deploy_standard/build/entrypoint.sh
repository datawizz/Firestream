#!/bin/bash

# Entrypoint

# A script to read a secret from Google Cloud Secret Manager and write it to a file.
# This script must be run as root to place the secret in the correct location.

# Exit on any error and pipe failures
set -e -o pipefail

# Use the embedded service account credentials to authenticate
gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS

# Access the latest version of the secret.
# The format is assumed to be a JSON string. Save it locally.
gcloud secrets versions access latest --secret deployment-secrets > /home/app/secrets.json


### Cloudflare Tunnel ###

# Config the cloudflared credentials.json
jq -r '.tunnel.credentials_b64' secrets.json | base64 -d > /home/app/credentials.json


# Config the cloudflared config.yaml
mkdir -p /etc/cloudflared/
jq -r '.tunnel.yaml_config_b64' secrets.json | base64 -d > /etc/cloudflared/config.yaml

# Start the cloudflared service
cloudflared tunnel run --credentials-file /home/app/credentials.json

### Tailscale ###

# Config the tailscale authkey
jq -r '.tailscale' secrets.json | base64 -d > /home/app/tailscale_authkey

# Start the tailscale service
tailscale up --authkey-file /home/app/tailscale_authkey
