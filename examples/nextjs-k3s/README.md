# Next.js on local k3s / k3d — a Firestream deployment (custom app)

A complete, copyable pattern for running a **net-new Next.js app** as a
Firestream supported app on a **local Kubernetes cluster** (host k3s or a
throwaway k3d), with **your app baked into the image**. Highlights:

- a custom Next.js app ([`app/`](./app)) compiled to **standalone output** and
  baked into the image at build time via `config.nextjs.vendoredApp` — no
  ConfigMap, git-sync, or PVC;
- the bundled **PostgreSQL** subchart running the **firestream-postgresql**
  image (injected by the flake);
- **inline credentials** from `config.nix` (no Secret Manager);
- **`local-path`** persistence and **ClusterIP + port-forward** access;
- the `firestream-*` images **side-loaded** into containerd (no registry).

Local-dev sibling of [`../nextjs-gke`](../nextjs-gke) and
[`../nextjs-docker-compose`](../nextjs-docker-compose). See
[`../README.md`](../README.md) for the general pattern.

## Layout

```
nextjs-k3s/
├── flake.nix                       # imports firestream; packages.{nextjs-chart,nextjs-image,postgresql-image}
├── config.nix                      # ← YOU EDIT THIS: namespace, storage, db passwords
├── app/                            # ← YOUR Next.js app, baked into the image
├── firestream/
│   ├── nextjs-overrides.nix        # sparse CHART override (bundled postgres creds, local-path)
│   └── nextjs-image-overrides.nix  # config.nextjs.vendoredApp = { src = ./app; npmDepsHash = ...; }
├── scripts/
│   └── deploy-local.sh             # build → side-load 2 images → helm upgrade --install
└── Makefile                        # chart / render / deploy / port-forward / status / destroy / clean
```

## How the app gets baked in

`firestream/nextjs-image-overrides.nix` sets
`config.nextjs.vendoredApp = { src = ./app; npmDepsHash = ...; }`, which the
nextjs container's `mkNodePackage` compiles to a Next.js standalone server and
bakes into the image. Because the chart override does **not** repoint the image,
the cluster runs this custom `firestream-nextjs:1.0.0` once `make deploy`
side-loads it. This is the Next.js analogue of airflow's vendored DAGs — *inject
preferences → custom image, no fork.*

The bundled PostgreSQL is wired to the app automatically: the chart sets
`PGHOST/PGPORT/PGUSER/PGDATABASE` from the subchart and `PGPASSWORD` from its
generated Secret.

## Quickstart

### 0. Prerequisites
`nix` (flakes), `docker`, `kubectl`, `helm`, and **one** of:
- a running **host k3s** where you can talk to containerd without sudo (you're
  in the `k3s` group — `id | grep k3s`); or
- a **k3d** cluster (`k3d cluster create nextjs`, then `export K3D_CLUSTER=nextjs`).

Everything else is in the dev shell — `nix develop` (or `direnv allow`).

### 1. (optional) Edit `config.nix`
Set the namespace and database passwords.

### 2. Build + inspect the chart (no cluster needed)
```bash
make chart
grep -E 'firestream-nextjs|firestream-postgresql|defaultStorageClass' result/values.yaml
```
Confirm the main image is `firestream-nextjs` and the bundled DB is
`firestream-postgresql`.

### 3. Deploy
```bash
make deploy        # builds 2 images, side-loads them, helm upgrade --install
make status        # wait until nextjs + postgresql are Ready
```

### 4. Reach the app
```bash
make port-forward  # localhost:3000 -> svc/nextjs
```
Open <http://127.0.0.1:3000>; the homepage renders the live `select version(), now()`
from firestream-postgresql.

### 5. Clean reinstall
```bash
make destroy && make deploy
```

## Notes & gotchas

- **Separate image stores.** Host k3s / k3d uses its own containerd, independent
  of Docker. `make deploy` re-imports every run, so re-run after editing the app.
- **First build is slow** (`next build` + one-time github fetch); cached after.
- **In-repo testing before merge** (the flake pins github main):
  ```bash
  NIX_OVERRIDE="--override-input firestream path:../.." make deploy
  ```
