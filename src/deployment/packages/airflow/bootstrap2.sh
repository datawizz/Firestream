/workspace/docker/k3d/bootstrap.sh

# helm repo add apache-airflow https://airflow.apache.org
# helm upgrade --install airflow apache-airflow/airflow --namespace airflow --create-namespace



### Airflow ###
# Enables: spark://spark-master:7077
# helm install airflow bitnami/airflow




### Airflow ###

helm install airflow oci://registry-1.docker.io/bitnamicharts/airflow \
  -f /workspace/src/deploy/packages/airflow/values.yaml
