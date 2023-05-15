

###############################################################################
### Services                                                                ###
###############################################################################

set -e


#TODO bring this repo into the project as a submodule
# Install Bitnami repo
helm repo add bitnami https://charts.bitnami.com/bitnami

sleep 3


### PostgreSQL ###

#  TODO use the high availability one
helm install postgresql bitnami/postgresql \
  --set global.postgresql.auth.username="$POSTGRES_USER" \
  --set global.postgresql.auth.password="$POSTGRES_PASSWORD" \
  --set global.postgresql.auth.database="$POSTGRES_DEFAULT_DB" \
  --set global.postgresql.service.ports.postgresql="$POSTGRES_PORT" 
  
## PGAdmin ##

cd /workspace/submodules/rowanruseler/helm-charts && \
helm install pgadmin charts/pgadmin4 \
  --set serverDefinitions.enabled=true \
  --set serverDefinitions.servers.firstServer.Name="Postgres" \
  --set serverDefinitions.servers.firstServer.Group="Servers" \
  --set serverDefinitions.servers.firstServer.Port=$POSTGRES_PORT \
  --set serverDefinitions.servers.firstServer.Username=$POSTGRES_USER \
  --set serverDefinitions.servers.firstServer.Host=$POSTGRES_URL \
  --set serverDefinitions.servers.firstServer.SSLMode="prefer" \
  --set serverDefinitions.servers.firstServer.MaintenanceDB=$POSTGRES_DEFAULT_DB





# Wait for all pods to be ready
echo "Waiting for pods to be ready"

# Start watching pods
(kubectl get pods --watch) &

# Wait for pods to be ready
kubectl wait --timeout=600s --for=condition=ready pods --all -n default

# Kill the watch command
kill %1