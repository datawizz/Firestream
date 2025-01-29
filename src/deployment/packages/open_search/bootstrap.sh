

### Open Search ###

export OPENSEARCH_HOST="opensearch-cluster-master.default.svc.cluster.local"
export OPENSEARCH_PORT="9200"
export OPENSEARCH_USERNAME='admin'
export OPENSEARCH_PASSWORD='admin'


# cd /workspace/submodules/the-firestream-company/opensearch-project-helm-charts/charts && \
# helm install opensearch opensearch
# TODO the configuration of OpenSearch is not trival.
# The default username and password are admin/admin, which are used here implicitly.
# --set something.something="$OPENSEARCH_USERNAME" \
# --set something.something="$OPENSEARCH_PASSWORD" \

# helm repo add opensearch https://opensearch-project.github.io/helm-charts/
# helm install opensearch-dashboard opensearch/opensearch-dashboards
# helm install opensearch opensearch/opensearch

# export CONTAINER_PORT=$(kubectl get pod --namespace default $POD_NAME -o jsonpath="{.spec.containers[0].ports[0].containerPort}")
#   echo "Visit http://127.0.0.1:8080 to use your application"
#   kubectl --namespace default port-forward $POD_NAME 8085:$CONTAINER_PORT
