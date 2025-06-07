
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
