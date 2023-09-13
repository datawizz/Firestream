
### AWS ###

# Used for connecting to S3 API via MinIO for clients that assume default AWS credentials
# such as Project Nessie

# TODO make this compatible with existing AWS credentials instead of creating new ones

# Create AWS directory if not exists
mkdir -p ~/.aws

# Create credentials file
cat > ~/.aws/credentials << EOF
[default]
aws_access_key_id = ${S3_LOCAL_ACCESS_KEY_ID}
aws_secret_access_key = ${S3_LOCAL_SECRET_ACCESS_KEY}
EOF

# Create config file
cat > ~/.aws/config << EOF
[default]
region = ${S3_LOCAL_DEFAULT_REGION}
s3 =
    endpoint_url = ${S3_LOCAL_ENDPOINT_URL}
EOF