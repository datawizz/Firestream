


export GOOGLE_APPLICATION_CREDENTIALS="/workspace/gcp.json"
export CLOUDSDK_CORE_PROJECT="pleasantrees-dwh"
export PULUMI_CONFIG_PASSPHRASE="trustme"

gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS

cd /workspace/src/scripts/pulumi/deploy_standard
source venv/bin/activate
pip install -r requirements.txt

pulumi login file://$PWD/.pulumi

#!/bin/bash

# Default Pulumi stack prefix
PULUMI_PREFIX="deploy_standard"

# Check if .env file exists
if [ ! -f .env ]; then
    echo "Error: .env file not found!"
    exit 1
fi

# Function to determine if a value should be treated as secret
is_secret_var() {
    local var_name=$1
    # List of keywords that indicate a secret value
    secret_keywords=("token" "key" "secret" "password" "api" "auth" "cert" "ssh" "private")

    local lowercase_name=$(echo "$var_name" | tr '[:upper:]' '[:lower:]')
    for keyword in "${secret_keywords[@]}"; do
        if [[ "$lowercase_name" == *"$keyword"* ]]; then
            return 0  # true - is secret
        fi
    done
    return 1  # false - not secret
}

# Read .env file line by line
while IFS= read -r line || [ -n "$line" ]; do
    # Skip empty lines and comments
    [[ $line =~ ^[[:space:]]*$ ]] && continue
    [[ $line =~ ^[[:space:]]*# ]] && continue

    # Parse variable and value
    if [[ $line =~ ^([^=]+)=(.*)$ ]]; then
        var_name="${BASH_REMATCH[1]}"
        var_value="${BASH_REMATCH[2]}"

        # Remove any surrounding quotes from value
        var_value="${var_value#\"}"
        var_value="${var_value%\"}"
        var_value="${var_value#\'}"
        var_value="${var_value%\'}"

        # Convert variable name to lowercase for Pulumi
        pulumi_name=$(echo "$var_name" | tr '[:upper:]' '[:lower:]')

        # Check if variable should be secret
        if is_secret_var "$var_name"; then
            echo "Setting secret config: $pulumi_name"
            pulumi config set --secret "${PULUMI_PREFIX}:${pulumi_name}" "$var_value"
        else
            echo "Setting config: $pulumi_name"
            pulumi config set "${PULUMI_PREFIX}:${pulumi_name}" "$var_value"
        fi
    fi
done < .env

echo "Pulumi configuration complete!"




pulumi preview -s main
