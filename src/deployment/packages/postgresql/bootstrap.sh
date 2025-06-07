

### PostgreSQL ###

# PostgreSQL
# cd /workspace/charts/postgresql && helm dependency build && \
# helm install postgresql -f /workspace/charts/postgresql/values.yaml /workspace/charts/postgresql/

# sh /workspace/src/services/persistence/postgresql/helm_install.sh
# -f /workspace/charts/postgresql/values.yaml





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


# Check if secret already exists
if ! kubectl get secret postgres-creds > /dev/null 2>&1; then
  temp_file=$(mktemp)
  echo "postgres_username=${POSTGRES_USER}" > ${temp_file}
  echo "postgres_password=${POSTGRES_PASSWORD}" >> ${temp_file}
  kubectl create secret generic postgres-creds --from-env-file="${temp_file}"
  rm ${temp_file}
fi
