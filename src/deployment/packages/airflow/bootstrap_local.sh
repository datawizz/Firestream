


#!/bin/bash


bash /workspace/bin/install_scripts/airflow-debian.sh


# Environment variables
AIRFLOW_HOME=${AIRFLOW_HOME:-"/opt/airflow"}
DATABASE_URL=${AIRFLOW_DATABASE_URI}
DAGS_FOLDER=${AIRFLOW_DAGS_FOLDER}
LOGS_FOLDER=${AIRFLOW_LOGS_FOLDER}
EXECUTOR="LocalExecutor"
SCHEDULER_MAX_THREADS=1
WEB_SERVER_HOST="0.0.0.0"
WEB_SERVER_PORT=${AIRFLOW_PORT:-8080}
PARALLELISM=1

mkdir -p ${AIRFLOW_HOME}

# Create or overwrite the airflow.cfg file
cat <<EOF > ${AIRFLOW_HOME}/airflow.cfg
[core]
executor = ${EXECUTOR}
sql_alchemy_conn = ${AIRFLOW__DATABASE__SQL_ALCHEMY_CONN}
dags_folder = ${DAGS_FOLDER}
parallelism = ${PARALLELISM}

[logging]
base_log_folder = ${LOGS_FOLDER}

[scheduler]
scheduler_max_threads = ${SCHEDULER_MAX_THREADS}

[webserver]
web_server_host = ${WEB_SERVER_HOST}
web_server_port = ${WEB_SERVER_PORT}
EOF

echo "Airflow configuration file is set up at ${AIRFLOW_HOME}/airflow.cfg"


airflow db migrate

