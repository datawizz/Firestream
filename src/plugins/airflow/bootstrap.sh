

helm install airflow oci://registry-1.docker.io/bitnamicharts/airflow \
    --set airflow.loadExamples=true \
    --set auth.username=admin \
    --set auth.password=admin \
    --set auth.fernetKey=my-fernet-key \
    --set auth.secretKey=my-secret-key