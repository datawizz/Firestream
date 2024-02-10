#!/bin/bash

# Exposes ports for all services in the project. Used only in development mode!

mkdir -p /workspace/logs

if [ "$DEPLOYMENT_MODE" != "development" ]; then
  echo "Invalid deployment_mode $DEPLOYMENT_MODE."
  echo "only 'development' mode is supported for exposing ports using kubectl!"
  exit 1
fi

# PostgreSQL
nohup kubectl port-forward --namespace default svc/postgresql 5432:5432 > /workspace/logs/port_forwards.log 2>&1 &

# PgAdmin
# POD_NAME=$(kubectl get pods --namespace default -l "app.kubernetes.io/name=pgadmin4,app.kubernetes.io/instance=pgadmin" -o jsonpath="{.items[0].metadata.name}")
# nohup kubectl port-forward --namespace default $POD_NAME 8080:80 > /workspace/logs/port_forwards.log 2>&1 &

# Nessie
nohup kubectl --namespace default port-forward svc/nessie 19120:19120 > /workspace/logs/port_forwards.log 2>&1 &

# Spark master
nohup kubectl port-forward --namespace default svc/spark-master-svc 8082:80 > /workspace/logs/port_forwards.log 2>&1 &

# MinIO
nohup kubectl port-forward --namespace default svc/minio 9001:9001 > /workspace/logs/port_forwards.log 2>&1 &

# Solr
nohup kubectl port-forward --namespace default svc/solr 8983:8983 > /workspace/logs/port_forwards.log 2>&1 &

# Solr Operator




# # Metabase
# POD_NAME=$(kubectl get pods --namespace default -l "app=metabase,release=metabase" -o jsonpath="{.items[0].metadata.name}")
# nohup kubectl port-forward --namespace default $POD_NAME 8081:3000 > /workspace/logs/port_forwards.log 2>&1 &

# Superset #
nohup kubectl port-forward --namespace default service/superset 8088:8088 > /workspace/logs/port_forwards.log 2>&1 &


# Jaeger UI #
nohup kubectl port-forward --namespace default service/jaeger-query 8089:16686 > /workspace/logs/port_forwards.log 2>&1 &

# Open Search #
nohup kubectl port-forward --namespace default service/opensearch-cluster-master 9200:9200 > /workspace/logs/port_forwards.log 2>&1 &

nohup kubectl port-forward --namespace default service/opensearch-dashboard-opensearch-dashboards 5601:5601 > /workspace/logs/port_forwards.log 2>&1 &