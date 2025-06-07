


# cd /workspace/submodules/the-firestream-company/superset/helm/superset && helm dependency build



### Superset ###
# cd /workspace/submodules/the-firestream-company/superset/helm/superset &&
#   helm dependency build && \
#   cd /workspace/submodules/the-firestream-company/superset/helm && \
#   helm install superset superset




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
