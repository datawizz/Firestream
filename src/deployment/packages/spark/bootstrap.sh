

### Spark Cluster ###
# Enables: spark://spark-master:7077
# helm install spark bitnami/spark #-f /workspace/charts/firestream/subcharts/spark_cluster/values.yaml



### Spark Operator ###
cd /workspace/submodules/the-firestream-company/spark-on-k8s-operator/charts/spark-operator-chart && \
helm upgrade --install spark-operator . --namespace "default" \
  --set sparkJobNamespace="default" \
  --set webhook.enable=true




#GCP kubectl create clusterrolebinding <user>-cluster-admin-binding --clusterrole=cluster-admin --user=<user>@<domain>



# $ helm install my-release spark-operator/spark-operator --namespace spark-operator --set sparkJobNamespace=test-ns



# retrieve config
# kubectl get sparkapplications spark-pi -o=yaml
# kubectl describe sparkapplication spark-pi
# apiVersion: sparkoperator.k8s.io/v1beta2
# kind: SparkApplication
# metadata:
#   ...
# spec:
#  deps: {}
#
