#!/bin/bash

###############################################################################
### Services                                                                ###
###############################################################################

# TODO ensure the configurations and environment variables are set correctly

set -e

#TODO bring this repo into the project as a submodule
# Install Bitnami repo
helm repo add bitnami https://charts.bitnami.com/bitnami

sleep 3



### Ingress ###

# Add Nginx
# https://kind.sigs.k8s.io/docs/user/ingress/#ingress-nginx
# Contains extra configs for KinD
# kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml --v=2

# kubectl wait --namespace ingress-nginx \
#   --for=condition=ready pod \
#   --selector=app.kubernetes.io/component=controller \
#   --timeout=90s



### Kafka ###
#helm install kafka bitnami/kafka -f /workspace/k8/kafka/values.yaml



### PostgreSQL ###

# PostgreSQL
# cd /workspace/charts/postgresql && helm dependency build && \
# helm install postgresql -f /workspace/charts/postgresql/values.yaml /workspace/charts/postgresql/

# sh /workspace/src/services/persistence/postgresql/helm_install.sh
# -f /workspace/charts/postgresql/values.yaml


# Build all services
#sh /workspace/opt/cicd_scripts/build.sh



# Commands to test the cluster via psql terminal
# export POSTGRES_PASSWORD=$(kubectl get secret --namespace default postgresql -o jsonpath="{.data.postgres-password}" | base64 -d) &&
    # PGPASSWORD="$POSTGRES_PASSWORD" psql --host postgresql.default.svc.cluster.local -U postgres -d postgres -p 5432

### PostgreSQL ###
#  -f /workspace/charts/postgresql/values.yaml \


#  TODO use the high availability one
helm install postgresql bitnami/postgresql \
  --set global.postgresql.auth.username="$POSTGRES_USER" \
  --set global.postgresql.auth.password="$POSTGRES_PASSWORD" \
  --set global.postgresql.auth.database="$POSTGRES_DEFAULT_DB" \
  --set global.postgresql.service.ports.postgresql="$POSTGRES_PORT"
  
## PGAdmin ##

# cd /workspace/submodules/rowanruseler/helm-charts && \
# helm install pgadmin charts/pgadmin4 \
#   --set serverDefinitions.enabled=true \
#   --set serverDefinitions.servers.firstServer.Name="Postgres" \
#   --set serverDefinitions.servers.firstServer.Group="Servers" \
#   --set serverDefinitions.servers.firstServer.Port=$POSTGRES_PORT \
#   --set serverDefinitions.servers.firstServer.Username=$POSTGRES_USER \
#   --set serverDefinitions.servers.firstServer.Host=$POSTGRES_URL \
#   --set serverDefinitions.servers.firstServer.SSLMode="prefer" \
#   --set serverDefinitions.servers.firstServer.MaintenanceDB=$POSTGRES_DEFAULT_DB



### Project Nessie ###

# Create a kubernetes secret for the postgres-creds credentials
# required (and best practice) for this helm chart

# Make a new temporary file
temp_file=$(mktemp)

# Write the values of the environment variables to the temp file
echo "postgres_username=${POSTGRES_USER}" > ${temp_file}
echo "postgres_password=${POSTGRES_PASSWORD}" >> ${temp_file}
cat ${temp_file}

# Create the secret from the temp file
kubectl create secret generic postgres-creds --from-env-file="${temp_file}"

# Delete the temp file
rm ${temp_file}

cd /workspace/submodules/the-fireworks-company/nessie && \
helm install nessie helm/nessie \
  --set versionStoreType=TRANSACTIONAL \
  --set postgres.jdbcUrl="$JDBC_CONNECTION_STRING" \
  --set image.tag="$NESSIE_VERSION"

### Spark Cluster ###
# Enables: spark://spark-master:7077
helm install spark bitnami/spark #-f /workspace/charts/fireworks/subcharts/spark_cluster/values.yaml

# ### MongoDB ###
# helm install mongodb bitnami/mongodb-sharded
# #-f /workspace/charts/mongodb/values.yaml


### Minio ###
helm install minio bitnami/minio -f /workspace/charts/fireworks/subcharts/minio/chart/values.yaml \
  --set auth.rootUser="$S3_LOCAL_ACCESS_KEY_ID" \
  --set auth.rootPassword="$S3_LOCAL_SECRET_ACCESS_KEY" \
  --set defaultBuckets="$S3_LOCAL_BUCKET_NAME"

# ### Solr ###
# helm install solr bitnami/solr -f /workspace/charts/solr/chart/values.yaml \
#   --set auth.adminUsername="$SOLR_USERNAME" \
#   --set auth.adminPassword="$SOLR_PASSWORD" \
#   --set coreNames="$SOLR_DEFAULT_CORE"

### Kafka ###
helm upgrade --install kafka bitnami/kafka --version 24.0.10  \
  --set controller.replicaCount=5 \
  --set controller.heapOpts="-Xmx1024m -Xms1024m" \
  --set controller.persistence.size=100Gi \
  --set listeners.client.protocol=PLAINTEXT \
  --set listeners.controller.protocol=PLAINTEXT \
  --set listeners.interbroker.protocol=PLAINTEXT \
  --set listeners.external.protocol=PLAINTEXT


### Kyuubi ###
cd /workspace/submodules/the-fireworks-company/kyuubi && \
helm install kyuubi charts/kyuubi


# cd /workspace/submodules/the-fireworks-company/superset/helm/superset && helm dependency build

### Superset ###
cd /workspace/submodules/the-fireworks-company/superset/helm/superset &&
  helm dependency build && \
  cd /workspace/submodules/the-fireworks-company/superset/helm && \
  helm install superset superset



### Open Search ###
# cd /workspace/submodules/the-fireworks-company/opensearch-project-helm-charts/charts && \
# helm install opensearch opensearch
# TODO the configuration of OpenSearch is not trival. 
# The default username and password are admin/admin, which are used here implicitly.
# --set something.something="$OPENSEARCH_USERNAME" \
# --set something.something="$OPENSEARCH_PASSWORD" \

helm repo add opensearch https://opensearch-project.github.io/helm-charts/
helm install opensearch-dashboard opensearch/opensearch-dashboards
helm install opensearch opensearch/opensearch

# export CONTAINER_PORT=$(kubectl get pod --namespace default $POD_NAME -o jsonpath="{.spec.containers[0].ports[0].containerPort}")
#   echo "Visit http://127.0.0.1:8080 to use your application"
#   kubectl --namespace default port-forward $POD_NAME 8085:$CONTAINER_PORT

### Airflow ###
# Enables: spark://spark-master:7077
helm install airflow bitnami/airflow




# ### Superset ###

# # Define the name of the secret, Helm release name, and namespace
# SECRET_NAME="superset-secret-config-py"
# NAMESPACE="default"

# #TODO make the namespace

# # Create a temporary file
# TEMP_FILE=$(mktemp)

# # Write the Python configuration to the temporary file
# #echo "SQLALCHEMY_DATABASE_URI = '${DATABASE_URL}'" > $TEMP_FILE

# echo "SQLALCHEMY_DATABASE_URI = 'postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_URL}:${POSTGRES_PORT}/superset_demo'" > $TEMP_FILE

# # Create or replace the secret with the contents of the temporary file
# kubectl -n $NAMESPACE delete secret $SECRET_NAME --ignore-not-found
# kubectl -n $NAMESPACE create secret generic $SECRET_NAME --from-file=superset_config.py=$TEMP_FILE

# # Remove the temporary file
# rm $TEMP_FILE


# helm install superset /workspace/submodules/apache/superset/helm/superset \
#     --set configFromSecret=superset-secret-config-py

export OPENSEARCH_HOST="opensearch-cluster-master.default.svc.cluster.local"
export OPENSEARCH_PORT="9200"
export OPENSEARCH_USERNAME='admin'
export OPENSEARCH_PASSWORD='admin'


# ### Jaeger All-in-One ###
# helm repo add jaegertracing https://jaegertracing.github.io/helm-charts
# helm upgrade --install jaeger jaegertracing/jaeger \
#   --set provisionDataStore.cassandra=false \
#   --set storage.type=elasticsearch \
#   --set storage.elasticsearch.host="$OPENSEARCH_HOST" \
#   --set storage.elasticsearch.port="$OPENSEARCH_PORT" \
#   --set storage.elasticsearch.user="$OPENSEARCH_USERNAME" \
#   --set storage.elasticsearch.password="$OPENSEARCH_PASSWORD" \
#   --set ingester.enabled=true \
#   --set storage.kafka.brokers[0]="$KAFKA_BOOTSTRAP_SERVERS" \
#   --set storage.kafka.topic="jaeger-spans" 
  # --debug --dry-run


  # --set provisionDataStore.cassandra=false \
  # --set provisionDataStore.elasticsearch=true \
  # --set storage.type=elasticsearch

# helm install jaeger jaegertracing/jaeger \
#   --set provisionDataStore.cassandra=false \
#   --set storage.type=elasticsearch \
#   --set storage.elasticsearch.host=<HOST> \
#   --set storage.elasticsearch.port=<PORT> \
#   --set storage.elasticsearch.user=<USER> \
#   --set storage.elasticsearch.password=<password>


# ### Metabase ###
# cd /workspace/charts/metabase/submodules/metabase-helm-chart && \
#   helm install metabase . -f /workspace/charts/metabase/values.yaml \
#   --set database.connectionURI="$METABASE_CONNECTION_URI"


# ### Hive Metastore ###
# cd /workspace/charts/hive_metastore \
#   && helm install hive-metastore . \
#   --set env.POSTGRES_URL="$POSTGRES_URL" \
#   --set env.POSTGRES_USER="$POSTGRES_USER" \
#   --set env.POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
#   --set env.POSTGRES_PORT="$POSTGRES_PORT" \
#   --set env.METASTORE_DB_NAME="$METASTORE_DB_NAME" \
#   --set env.METASTORE_HOME="$METASTORE_HOME" \
#   --set env.JDBC_CONNECTION_STRING="$JDBC_CONNECTION_STRING"

# ### Websocket Middleware ###
# cd /workspace/services/javascript/websocket_middleware/chart && helm install websocket-middleware -f values.yaml .

# ### Dashboard ###
# cd /workspace/services/javascript/dashboard/chart && helm install dashboard -f values.yaml .


echo "Waiting for pods to be ready"

# Define the watch command
watch_cmd="kubectl get pods --watch -n default"

# Run the watch command in the background
${watch_cmd} &

# Save the PID of the watch command
watch_pid=$!

# Wait for pods to be ready excluding the 'init-db' pod
kubectl get pods --no-headers -n default | awk '!/init-db/{print $1}' | while read pod; do
  # Wait for each pod to be ready and print status
  if kubectl wait --timeout=600s --for=condition=ready pod/$pod -n default; then
    echo "Pod $pod is ready"
  else
    echo "Timeout while waiting for pod $pod to be ready"
  fi
done

# Kill the watch command
kill ${watch_pid}
