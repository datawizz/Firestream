#!/bin/bash

# Set Helm values.yaml using environment variables
AUTH_ENCRYPT_SECRET_KEY="SOME_RANDOM_SECRET_KEY_THAT_IS_ALPHA_NUMERIC"

S3_DOMAIN_NAME="*.s3.local.lakefs.io" # TODO change to your domain name

helm uninstall lakefs

# TODO enable SSL for S3

cat <<EOF > values.yaml
secrets:
  databaseConnectionString: "$DATABASE_URL"
  authEncryptSecretKey: "$AUTH_ENCRYPT_SECRET_KEY"
lakefsConfig: |
  database:
    type: postgres
  blockstore:
    type: s3
    s3:
      region: "$S3_LOCAL_DEFAULT_REGION"
      endpoint: "$S3_LOCAL_ENDPOINT_URL"
      skip_verify_certificate_test_only: true
      credentials:
        access_key_id: "$S3_LOCAL_ACCESS_KEY_ID"
        secret_access_key: "$S3_LOCAL_SECRET_ACCESS_KEY"
      force_path_style: true
      discover_bucket_region: false
  gateways:
    s3:
      domain_name: "$S3_DOMAIN_NAME"
      region: "$S3_LOCAL_DEFAULT_REGION"
EOF

helm repo add lakefs https://charts.lakefs.io
helm install -f /workspace/src/plugins/lakefs/values.yaml lakefs lakefs/lakefs --version 0.9.23



