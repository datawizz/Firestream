# Airflow on local k3s / k3d — a Firestream deployment (custom baked DAG)

A complete, copyable pattern for running the Firestream **Airflow** app on a
**local Kubernetes cluster** (host k3s or a throwaway k3d), with a **custom DAG
baked into the image**. Highlights:

- a custom DAG ([`dags/hello_firestream.py`](./dags/hello_firestream.py)) baked
  into `/opt/firestream/airflow/dags` at build time via
  `config.airflow.vendoredDags` — no ConfigMap, git-sync, or PVC;
- the bundled **PostgreSQL** (metadata DB) and **Redis** (Celery broker)
  subcharts;
- **CeleryExecutor** (scheduler + worker + triggerer + dag-processor +
  api-server);
- **inline credentials** from `config.nix` (no Secret Manager);
- **`local-path`** persistence and **ClusterIP + port-forward** access;
- the `firestream-*` images **side-loaded** into containerd (no registry).

Local-dev sibling of [`../airflow-gke`](../airflow-gke) and
[`../airflow-docker-compose`](../airflow-docker-compose). See
[`../README.md`](../README.md) for the general pattern.

## Layout

```
airflow-k3s/
├── flake.nix                       # imports firestream; packages.{airflow-chart,airflow-image,postgresql-image,redis-image}
├── config.nix                      # ← YOU EDIT THIS: namespace, storage, passwords, crypto keys
├── dags/
│   └── hello_firestream.py         # ← the custom DAG, baked into the image
├── firestream/
│   ├── airflow-overrides.nix       # sparse CHART override (CeleryExecutor, local-path, inline creds)
│   └── airflow-image-overrides.nix # config.airflow.vendoredDags = [ { src = ../dags; } ]
├── scripts/
│   └── deploy-local.sh             # build → side-load 3 images → helm upgrade --install
└── Makefile                        # chart / render / deploy / port-forward / status / destroy / clean
```

## How the DAG gets baked in

`firestream/airflow-image-overrides.nix` sets
`config.airflow.vendoredDags = [ { name = "example-dags"; src = ../dags; } ]`,
which the Airflow container's `vendor-dags.nix` lays into
`/opt/firestream/airflow/dags` inside the image. Because the chart override does
**not** repoint the image, the cluster runs this custom `firestream-airflow:3.0.3`
once `make deploy` side-loads it. The pods mount nothing over the dags folder, so
the baked file is exactly what the scheduler scans. This is the Airflow analogue
of odoo's vendored addons — *inject preferences → custom image, no fork.*

## Quickstart

### 0. Prerequisites
`nix` (flakes), `docker`, `kubectl`, `helm`, and **one** of:
- a running **host k3s** where you can talk to containerd without sudo (you're
  in the `k3s` group — `id | grep k3s`); or
- a **k3d** cluster (`k3d cluster create airflow`, then `export K3D_CLUSTER=airflow`).

Everything else is in the dev shell — `nix develop` (or `direnv allow`).

### 1. (optional) Edit `config.nix`
Set the namespace, passwords, and crypto keys. Generate a real Fernet key with
`python3 -c "from cryptography.fernet import Fernet; print(Fernet.generate_key().decode())"`.

### 2. Build + inspect the chart (no cluster needed)
```bash
make chart
grep -E 'firestream-airflow|executor:|defaultStorageClass' result/values.yaml
```
Confirm the image repo is `firestream-airflow` (not repointed) and
`executor: CeleryExecutor`.

### 3. Deploy
```bash
make deploy        # builds 3 images, side-loads them, helm upgrade --install
make status        # wait until all pods are Ready (Celery → several pods)
```

### 4. Reach Airflow + see the DAG
```bash
make port-forward  # localhost:8080 -> svc/airflow-web
```
Open <http://127.0.0.1:8080>, log in `admin` / `admin1234`, and confirm
**`hello_firestream`** is in the DAG list. Unpause + trigger it; the task logs
"Hello from a Firestream-baked DAG!".

### 5. Clean reinstall
```bash
make destroy && make deploy
```

## Notes & gotchas

- **Celery footprint.** Several pods (api-server, scheduler, worker, triggerer,
  dag-processor, postgresql, redis). Fine on a dev box; for a tiny machine,
  switch `executor = "LocalExecutor"` in the override (drops redis + worker).
- **Separate image stores.** Host k3s / k3d uses its own containerd, independent
  of Docker. Rebuilding + `docker load` does not change what the cluster runs
  until re-imported — `make deploy` does this every run, so re-run after editing
  a DAG.
- **First build is slow** (large Airflow image + one-time github fetch); cached after.
- **In-repo testing before merge** (the flake pins github main):
  ```bash
  NIX_OVERRIDE="--override-input firestream path:../.." make deploy
  ```
