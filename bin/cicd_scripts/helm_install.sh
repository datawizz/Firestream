

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
helm install postgresql bitnami/postgresql
  # --set global.postgresql.username="$POSTGRES_USER" \
  # --set global.postgresql.password="N1mSIBcsYg" \
  # --set global.postgresql.database="regmgr" \
  # --set fullnameOverride="postgresql"





### Spark ###
helm install spark bitnami/spark -f /workspace/charts/spark/values.yaml


### Minio ###
helm install minio bitnami/minio -f /workspace/charts/minio/chart/values.yaml \
  --set auth.rootUser="$S3_ACCESS_KEY_ID" \
  --set auth.rootPassword="$S3_SECRET_ACCESS_KEY" \
  --set defaultBuckets="$S3_BUCKET_NAME"

### Solr ###
helm install solr bitnami/solr -f /workspace/charts/solr/chart/values.yaml \
  --set auth.adminUsername="$SOLR_USERNAME" \
  --set auth.adminPassword="$SOLR_PASSWORD" \
  --set coreNames="$SOLR_DEFAULT_CORE"

### Kafka ###
helm install kafka bitnami/kafka -f /workspace/charts/kafka/chart/values.yaml


### Jupyter Hub ###
helm install jupyter bitnami/jupyterhub -f /workspace/charts/jupyterhub/values.yaml



# ### Websocket Middleware ###
# cd /workspace/services/javascript/websocket_middleware/chart && helm install websocket-middleware -f values.yaml .

# ### Dashboard ###
# cd /workspace/services/javascript/dashboard/chart && helm install dashboard -f values.yaml .



# Wait for all pods to be ready
echo "Waiting for pods to be ready"
kubectl wait --timeout=120s --for=condition=ready pods --all -n default