"""Pulumi stack for the Firestream Airflow GKE example.

Provisions, in one Google Cloud project, everything the deployment needs:

  * the required service APIs
  * a GKE Autopilot cluster (VPC-native, Workload Identity enabled by default)
  * an Artifact Registry Docker repository for the firestream-* images
  * Secret Manager secrets holding the Airflow admin/crypto + Postgres + Redis
    credentials
  * a deployer service account with the IAM Cloud Build needs

Secret *values* come from the stack config (set with `pulumi config --secret`)
so they never live in source. Cloud Build later reads them back from Secret
Manager and materialises the in-cluster Kubernetes Secrets the Helm chart
references via `existingSecret` (see ../scripts/sync-secrets.sh).

Stack outputs (clusterName / region / arHost / arRepo) are copied into
../config.nix to wire the Nix chart override to this infrastructure.
"""

import pulumi
import pulumi_gcp as gcp

cfg = pulumi.Config()
gcp_cfg = pulumi.Config("gcp")

project = gcp_cfg.require("project")
region = gcp_cfg.get("region") or "us-central1"
cluster_name = cfg.get("clusterName") or "firestream-airflow"
ar_repo_id = cfg.get("arRepo") or "firestream"

# --- 1. Enable the APIs this stack depends on -------------------------------
service_apis = [
    "container.googleapis.com",
    "artifactregistry.googleapis.com",
    "secretmanager.googleapis.com",
    "cloudbuild.googleapis.com",
]
services = {
    api: gcp.projects.Service(
        api.split(".")[0],
        project=project,
        service=api,
        disable_on_destroy=False,
    )
    for api in service_apis
}

# --- 2. GKE Autopilot cluster -----------------------------------------------
# Autopilot clusters are VPC-native and have Workload Identity + container-
# native (NEG) load balancing enabled automatically, which is what the
# ClusterIP-service + GCE-ingress shape in the chart override relies on.
cluster = gcp.container.Cluster(
    "firestream-airflow",
    name=cluster_name,
    project=project,
    location=region,
    enable_autopilot=True,
    deletion_protection=False,
    opts=pulumi.ResourceOptions(
        depends_on=[services["container.googleapis.com"]],
    ),
)

# --- 3. Artifact Registry (Docker) ------------------------------------------
repo = gcp.artifactregistry.Repository(
    "firestream",
    project=project,
    location=region,
    repository_id=ar_repo_id,
    format="DOCKER",
    description="Firestream container images for the Airflow deployment",
    opts=pulumi.ResourceOptions(
        depends_on=[services["artifactregistry.googleapis.com"]],
    ),
)

# --- 4. Secret Manager secrets ----------------------------------------------
# The KEY NAMES here are the GCP secret ids; sync-secrets.sh maps them to the
# Kubernetes Secret keys the chart expects (airflow-password / airflow-fernet-key
# / airflow-secret-key / airflow-jwt-secret-key / password / postgres-password /
# redis-password).
secret_specs = {
    "airflow-password": cfg.require_secret("airflowPassword"),
    "airflow-fernet-key": cfg.require_secret("fernetKey"),
    "airflow-secret-key": cfg.require_secret("secretKey"),
    "airflow-jwt-secret-key": cfg.require_secret("jwtSecretKey"),
    "airflow-db-password": cfg.require_secret("dbPassword"),
    "airflow-db-postgres-password": cfg.require_secret("dbPostgresPassword"),
    "airflow-redis-password": cfg.require_secret("redisPassword"),
}


def make_secret(secret_id: str, value: pulumi.Output) -> gcp.secretmanager.Secret:
    secret = gcp.secretmanager.Secret(
        secret_id,
        project=project,
        secret_id=secret_id,
        replication=gcp.secretmanager.SecretReplicationArgs(
            auto=gcp.secretmanager.SecretReplicationAutoArgs(),
        ),
        opts=pulumi.ResourceOptions(
            depends_on=[services["secretmanager.googleapis.com"]],
        ),
    )
    gcp.secretmanager.SecretVersion(
        f"{secret_id}-version",
        secret=secret.id,
        secret_data=value,
    )
    return secret


secrets = {sid: make_secret(sid, val) for sid, val in secret_specs.items()}

# --- 5. Deployer service account + IAM for Cloud Build ----------------------
deployer = gcp.serviceaccount.Account(
    "airflow-deployer",
    project=project,
    account_id="firestream-airflow-deployer",
    display_name="Firestream Airflow deployer (Cloud Build)",
)

deployer_member = deployer.email.apply(lambda email: f"serviceAccount:{email}")

deployer_roles = [
    "roles/container.developer",        # get-credentials + deploy to GKE
    "roles/artifactregistry.writer",    # push firestream-* images
    "roles/secretmanager.secretAccessor",  # read secret values at deploy time
]
for role in deployer_roles:
    gcp.projects.IAMMember(
        f"deployer-{role.rsplit('/', 1)[-1]}",
        project=project,
        role=role,
        member=deployer_member,
    )

# --- Outputs (copy into ../config.nix) --------------------------------------
pulumi.export("clusterName", cluster.name)
pulumi.export("region", region)
pulumi.export("arHost", f"{region}-docker.pkg.dev")
pulumi.export("arRepo", pulumi.Output.concat(project, "/", repo.repository_id))
pulumi.export("deployerServiceAccount", deployer.email)
pulumi.export("secretIds", list(secrets.keys()))
