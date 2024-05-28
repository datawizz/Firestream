import os
import subprocess
from kubernetes import client, config

# Name of the namespace, PVC and local directory with DAGs
namespace = "default"
pvc_name = "my-volume-claim"
local_dags_directory = "/path/to/your/local/dags"

def create_pvc():
    config.load_kube_config()
    v1 = client.CoreV1Api()

    pvc = client.V1PersistentVolumeClaim(
        api_version="v1",
        kind="PersistentVolumeClaim",
        metadata=client.V1ObjectMeta(name=pvc_name),
        spec=client.V1PersistentVolumeClaimSpec(
            access_modes=["ReadWriteMany"],
            resources=client.V1ResourceRequirements(
                requests={"storage": "1Gi"}
            )
        )
    )

    v1.create_namespaced_persistent_volume_claim(namespace, pvc)

def copy_dags_to_pvc():
    command = f"kubectl cp {local_dags_directory} {namespace}/{pvc_name}:/"
    subprocess.run(command, shell=True, check=True)

def update_helm_chart():
    command = "helm upgrade --install airflow apache-airflow/airflow \
                --set dags.persistence.enabled=true \
                --set dags.persistence.existingClaim=my-volume-claim \
                --set dags.gitSync.enabled=false"
    subprocess.run(command, shell=True, check=True)

def main():
    create_pvc()
    copy_dags_to_pvc()
    update_helm_chart()

if __name__ == "__main__":
    main()
