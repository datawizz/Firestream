


#!/bin/bash
# $BITNAMI_CHARTS_HOME=~/bitnami-charts

RELEASE_NAME="airflow"

cd ~/bitnami-charts/bitnami/airflow

#TODO bitnami packages dependencies as URLs are point back to the repo
# this could be improved by pointing to the local directory
# but this probably doesnt matter because the remote charts versions are locked
helm dependency build

helm install $RELEASE_NAME . \
    -f /workspace/src/deployment/packages/airflow/values.yaml




# Forward ports

kubectl port-forward svc/airflow 8080:8080 --namespace airflow




# # TODO Version lock
# # helm search repo bitnami/airflow --versions
# # --version 16.5.3 \
# # helm install airflow oci://registry-1.docker.io/bitnamicharts/airflow --version 16.1.5 \
# #     -f /workspace/src/plugins/airflow/values.yaml

# kubectl create secret generic my-webserver-secret --from-literal="webserver-secret-key=$(python3 -c 'import secrets; print(secrets.token_hex(16))')"

# helm repo add apache-airflow https://airflow.apache.org
# helm upgrade --install airflow apache-airflow/airflow --namespace default --create-namespace \
#     --set webserverSecretKey=$(kubectl get secret my-webserver-secret -o jsonpath="{.data.webserver-secret-key}" | base64 --decode) \
#     -f /workspace/src/plugins/airflow/com.values.yaml

# # NAME: airflow
# # LAST DEPLOYED: Sat Feb  3 19:19:15 2024
# # NAMESPACE: airflow
# # STATUS: deployed
# # REVISION: 1
# # TEST SUITE: None
# # NOTES:
# # Thank you for installing Apache Airflow 2.7.1!

# # Your release is named airflow.
# # You can now access your dashboard(s) by executing the following command(s) and visiting the corresponding port at localhost in your browser:

# # Airflow Webserver:     kubectl port-forward svc/airflow-webserver 8080:8080 --namespace airflow
# # Default Webserver (Airflow UI) Login credentials:
# #     username: admin
# #     password: admin
# # Default Postgres connection credentials:
# #     username: postgres
# #     password: postgres
# #     port: 5432

# # You can get Fernet Key value by running the following:

# #     echo Fernet Key: $(kubectl get secret --namespace airflow airflow-fernet-key -o jsonpath="{.data.fernet-key}" | base64 --decode)

# # ###########################################################
# # #  WARNING: You should set a static webserver secret key  #
# # ###########################################################

# # You are using a dynamically generated webserver secret key, which can lead to
# # unnecessary restarts of your Airflow components.

# # Information on how to set a static webserver secret key can be found here:
# # https://airflow.apache.org/docs/helm-chart/stable/production-guide.html#webserver-secret-key
