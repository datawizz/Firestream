# Airflow on GKE — a Firestream deployment (custom baked DAG)

A complete, copyable pattern for running the Firestream **Airflow** app on
**Google Kubernetes Engine**, with a **custom DAG baked into the image**, plus:

- **Pulumi (Python)** provisioning a GKE Autopilot cluster, an Artifact Registry
  repo, GCP Secret Manager secrets, and the IAM Cloud Build needs;
- **GCP Secret Manager** holding the Airflow admin/crypto + database + redis
  credentials, materialised into Kubernetes Secrets at deploy time (the chart
  references them by name via `existingSecret` — no secrets in git or
  `values.yaml`);
- **Cloud Build** building and pushing the `firestream-*` images (including the
  custom Airflow image with the baked DAG), building your customized chart
  bundle, and deploying it.

Anchored by one `flake.nix` that imports Firestream and two sparse override
modules (chart + image) — see [`../README.md`](../README.md) for the pattern.

## Layout

```
airflow-gke/
├── flake.nix                       # imports firestream; packages.airflow-chart = charts.airflow.eval(...).chartBundle
├── config.nix                      # ← YOU EDIT THIS: project, region, cluster, AR, domain, storage, secret names
├── dags/
│   └── hello_firestream.py         # ← the custom DAG, baked into the image
├── firestream/
│   ├── airflow-overrides.nix       # sparse CHART override (image refs, ingress, persistence, existingSecret)
│   └── airflow-image-overrides.nix # config.airflow.vendoredDags = [ { src = ../dags; } ]
├── pulumi/                         # GKE + Artifact Registry + Secret Manager + IAM (Python)
├── cloudbuild.yaml                 # build 3 images → build chart → auth GKE → sync secrets → deploy
├── scripts/
│   ├── sync-secrets.sh             # GCP Secret Manager → Kubernetes Secrets (idempotent)
│   └── deploy-local.sh             # the full pipeline, locally, for the dev loop
└── Makefile                        # chart / render / pulumi-* / deploy / destroy
```

## What gets overridden

`firestream/airflow-overrides.nix` sets only deployment-specific values on top of
the Firestream defaults:

| Override | Why |
|----------|-----|
| `auth.existingSecret`, `postgresql.auth.existingSecret`, `redis.auth.existingSecret` | Read credentials from K8s Secrets (sourced from Secret Manager) |
| `image.*`, `postgresql.image.*`, `redis.image.*` | Pull the `firestream-*` images from your Artifact Registry |
| `executor = "CeleryExecutor"` | Production-like topology (scheduler + worker + triggerer + dag-processor + api-server) |
| `*.persistence.*`, `global.defaultStorageClass` | Bind DB/broker/component PVCs to a GKE StorageClass |
| `ingress.*` (class `gce`) | Expose the Airflow web UI through a GKE L7 load balancer |

Credentials map to these Secret keys (Bitnami conventions):

- `airflow-credentials` → keys `airflow-password`, `airflow-fernet-key`, `airflow-secret-key`, `airflow-jwt-secret-key`
- `airflow-db-credentials` → keys `password` (app user) + `postgres-password` (admin)
- `airflow-redis-credentials` → key `redis-password`

## Quickstart

### 0. Prerequisites
`gcloud` (authenticated: `gcloud auth login` + `gcloud auth application-default
login`), `nix` (flakes enabled), and `docker`. The dev shell provides the rest —
`nix develop` (or `direnv allow`).

### 1. Provision infrastructure with Pulumi
```bash
cd pulumi
python -m venv venv && . venv/bin/activate && pip install -r requirements.txt
pulumi stack init dev
pulumi config set gcp:project   YOUR_PROJECT
pulumi config set gcp:region    us-central1
pulumi config set clusterName   firestream-airflow
pulumi config set arRepo        firestream
pulumi config set --secret airflowPassword     "$(openssl rand -base64 24)"
pulumi config set --secret fernetKey           "$(python3 -c 'from cryptography.fernet import Fernet; print(Fernet.generate_key().decode())')"
pulumi config set --secret secretKey           "$(openssl rand -base64 24)"
pulumi config set --secret jwtSecretKey        "$(openssl rand -base64 24)"
pulumi config set --secret dbPassword          "$(openssl rand -base64 24)"
pulumi config set --secret dbPostgresPassword  "$(openssl rand -base64 24)"
pulumi config set --secret redisPassword       "$(openssl rand -base64 24)"
pulumi up
```

### 2. Wire `config.nix`
```bash
pulumi stack output       # clusterName, region, arHost, arRepo
```
Set `projectId`, `region`, `clusterName`, `arHost`, `arRepo`, `domain`, and
`storageClass` to match.

### 3. Build + inspect the chart (no cluster needed)
```bash
make chart      # nix build .#airflow-chart
grep -E 'existingSecret|registry|executor:|defaultStorageClass' result/values.yaml
```
Confirm `existingSecret: airflow-credentials`, the image `registry` points at your
Artifact Registry host, and `executor: CeleryExecutor`.

### 4. Deploy
Locally:
```bash
make deploy     # scripts/deploy-local.sh: push 3 images, sync secrets, helm deploy
```
Or via Cloud Build (connect `cloudbuild.yaml` to a trigger, substitutions
matching `config.nix`):
```bash
gcloud builds submit --config cloudbuild.yaml \
  --substitutions=_REGION=us-central1,_CLUSTER=firestream-airflow,_NAMESPACE=airflow,_AR_HOST=us-central1-docker.pkg.dev,_AR_REPO=YOUR_PROJECT/firestream,_IMAGE_TAG=latest
```

### 5. Reach Airflow + see the DAG
```bash
kubectl -n airflow get pods -w
kubectl -n airflow get ingress      # note the LB address; point your DNS at it
```
Log in as `admin` (password from Secret Manager) and confirm the
`hello_firestream` DAG is listed.

## The baked DAG (custom image)

This example builds a **custom Airflow image** by injecting preferences into
Firestream's Nix system — *inject preferences → get a custom image, no fork.*

| Seam | File | Customizes |
|------|------|------------|
| `charts.airflow.eval` | `firestream/airflow-overrides.nix` | Helm **chart** values (image refs, ingress, persistence, secrets) |
| `images.airflow.eval` | `firestream/airflow-image-overrides.nix` | The **container image** (baked DAGs, env, …) |

`firestream/airflow-image-overrides.nix` sets
`config.airflow.vendoredDags = [ { name = "example-dags"; src = ../dags; } ]`,
which lays every file under [`dags/`](./dags) into `/opt/firestream/airflow/dags`
(Airflow's `dags_folder`). `nix build .#airflow-image` produces the image with
the DAG baked in; Cloud Build pushes it as `<AR>/firestream-airflow:<tag>` and
`airflow-overrides.nix` points the chart at that same ref — so the cluster runs
the customized image. Add more `.py` files to `dags/` (helpers welcome) and
rebuild. You can also vendor a git repo of DAGs:

```nix
vendoredDags = [
  { name = "my-dags"; owner = "me"; repo = "airflow-dags";
    rev = "<commit-sha>"; hash = "sha256-…"; sourceRoot = "dags"; }
];
```
**Getting the `hash`:** start from a wrong value, run `nix build .#airflow-image`,
and copy the `got: sha256-…` from the error.

## Notes & follow-ups

- **Image-registry override**: the chart's default image slots are injected by
  Firestream (`docker.io/firestream-*`). The override repoints them at Artifact
  Registry; an explicit user value wins. Verify in `result/values.yaml` after
  `make chart`.
- **TLS / managed certs / DNS**: not wired here. Add a `ManagedCertificate` +
  `networking.gke.io/managed-certificates` ingress annotation and an A record for
  `config.nix:domain` as a follow-up.
- **External DB / broker**: this example uses the bundled in-cluster Postgres +
  Redis subcharts. To use Cloud SQL / Memorystore instead, set
  `postgresql.enabled = false` / `redis.enabled = false` and wire
  `externalDatabase.*` / `externalRedis.*`.
- **Trigger wiring**: connecting `cloudbuild.yaml` to your repo is a one-time
  console step; Pulumi already grants the Cloud Build SA the needed roles.
