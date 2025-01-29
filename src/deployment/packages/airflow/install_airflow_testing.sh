#!/bin/bash

# Default versions and installation paths
AIRFLOW_VERSION="2.10.3"
AIRFLOW_HOME="/opt/airflow"
AIRFLOW_SHA512="94b576176b4ba6f9e90fe82ddce2aa198a2c85685a4a3faa844ba5121a25d6189e52feab66df73859152503ada98f6507e7e836f09a0924777a380817f1e8ea5"

# Function to verify SHA512 hash
verify_sha512() {
    local file=$1
    local expected_hash=$2
    local computed_hash=$(sha512sum "$file" | awk '{print $1}')

    if [ "$computed_hash" != "$expected_hash" ]; then
        echo "Hash verification failed for $file"
        echo "Expected: $expected_hash"
        echo "Got: $computed_hash"
        return 1
    fi
    echo "Hash verification passed for $file"
    return 0
}

# Install system dependencies
echo "Installing system dependencies..."
apt-get update && export DEBIAN_FRONTEND=noninteractive
apt-get install -y --no-install-recommends \
    rsync

# Create Airflow user and group
echo "Creating Airflow user..."
groupadd -r airflow
useradd -r -g airflow airflow

# Create necessary directories
echo "Creating directories..."
mkdir -p "${AIRFLOW_HOME}"
mkdir -p /tmp/downloads

# Change to downloads directory
cd /tmp/downloads

# Download and install Airflow
echo "Downloading Apache Airflow ${AIRFLOW_VERSION}..."
wget "https://dlcdn.apache.org/airflow/${AIRFLOW_VERSION}/apache-airflow-${AIRFLOW_VERSION}-source.tar.gz"

echo "Verifying Airflow download..."
verify_sha512 "apache-airflow-${AIRFLOW_VERSION}-source.tar.gz" "${AIRFLOW_SHA512}"

echo "Extracting Airflow..."
tar xf "apache-airflow-${AIRFLOW_VERSION}-source.tar.gz" -C "${AIRFLOW_HOME}"

# Install Python dependencies
echo "Installing Python dependencies..."
python3 -m pip install --upgrade pip setuptools wheel
python3 -m pip install apache-airflow==${AIRFLOW_VERSION} --constraint "https://raw.githubusercontent.com/apache/airflow/constraints-${AIRFLOW_VERSION}/constraints-3.11.txt"

# Set up Airflow environment variables
echo "Setting up environment variables..."
cat >> /etc/profile.d/airflow.sh << 'EOF'
export AIRFLOW_HOME=/opt/airflow
export PATH=$PATH:$AIRFLOW_HOME/bin
EOF

# Set correct permissions
echo "Setting permissions..."
chown -R airflow:airflow "${AIRFLOW_HOME}"

# Initialize Airflow database
echo "Initializing Airflow database..."
su - airflow -c "export AIRFLOW_HOME=${AIRFLOW_HOME} && airflow db init"

# Create default Airflow configuration
echo "Creating default Airflow configuration..."
cat > ${AIRFLOW_HOME}/airflow.cfg << 'EOF'
[core]
dags_folder = ${AIRFLOW_HOME}/dags
base_log_folder = ${AIRFLOW_HOME}/logs
logging_level = INFO
executor = LocalExecutor
sql_alchemy_conn = sqlite:///${AIRFLOW_HOME}/airflow.db
load_examples = False

[webserver]
web_server_host = 0.0.0.0
web_server_port = 8080
rbac = True

[scheduler]
dag_file_processor_timeout = 600
EOF

# Clean up
echo "Cleaning up..."
cd /
rm -rf /tmp/downloads
apt-get clean
rm -rf /var/lib/apt/lists/*

echo "Installation complete!"
echo "Please source the environment variables:"
echo "source /etc/profile.d/airflow.sh"
echo ""
echo "To start Airflow services:"
echo "1. Start the webserver: airflow webserver -D"
echo "2. Start the scheduler: airflow scheduler -D"
echo ""
echo "Default Airflow UI will be available at: http://localhost:8080"
echo "Default username: admin"
echo "To create an admin user, run: airflow users create --role Admin --username admin --firstname FIRST_NAME --lastname LAST_NAME --email EMAIL --password PASSWORD"
