


helm repo add nessie https://charts.projectnessie.org

helm install nessie nessie/nessie --version "$NESSIE_VERSION" \
  --set versionStoreType=TRANSACTIONAL \
  --set postgres.jdbcUrl="$JDBC_CONNECTION_STRING" \
  --set image.tag="$NESSIE_VERSION"
