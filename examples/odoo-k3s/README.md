# Odoo on local k3s / k3d — a Firestream deployment

A complete, copyable pattern for running the Firestream **Odoo** app on a
**local Kubernetes cluster** (host k3s or a throwaway k3d), with:

- the bundled in-cluster **PostgreSQL** subchart (no external database);
- **inline credentials** from `config.nix` (no Secret Manager);
- **`local-path`** persistence (the default k3s/k3d provisioner);
- **ClusterIP + `kubectl port-forward`** access (no ingress controller needed);
- the `firestream-*` images **side-loaded** into the cluster's containerd —
  because they're Nix-built and live only on your machine, not a registry.

This is the local-dev sibling of [`../odoo-gke`](../odoo-gke): the *same* app and
the *same* `charts.odoo.eval` seam, shipped to a laptop instead of GKE. See
[`../README.md`](../README.md) for the general one-flake pattern.

## Layout

```
odoo-k3s/
├── flake.nix                  # imports firestream; packages.{odoo-chart,odoo-image,postgresql-image}
├── config.nix                 # ← YOU EDIT THIS: namespace, storage class, passwords, vendored addons
├── firestream/
│   ├── odoo-overrides.nix     # sparse CHART override (local-path, ClusterIP, inline creds)
│   └── odoo-image-overrides.nix  # vendored addons → custom firestream-odoo:18.0 image
├── scripts/
│   └── deploy-local.sh        # build → side-load images → helm upgrade --install
└── Makefile                   # chart / render / deploy / port-forward / status / destroy / clean
```

## What gets overridden

`firestream/odoo-overrides.nix` sets only what's specific to a local cluster on
top of the Firestream defaults — and **notably leaves the image refs alone** so
the cluster runs the images we side-load:

| Override | Why |
|----------|-----|
| `odooEmail`, `odooPassword`, `postgresql.auth.{password,postgresPassword}` | Inline credentials (no Secret Manager); explicit so `helm upgrade` is idempotent |
| `persistence.storageClass`, `postgresql.primary.persistence.storageClass` = `local-path` | Bind to the default k3s/k3d provisioner |
| `service.type = ClusterIP`, `ingress.enabled = false` | Reach Odoo via port-forward; no LB/ingress |
| `resourcesPreset = "small"` | Lighter footprint for a laptop |
| *(image refs unchanged)* | Keep the default-injected `firestream-odoo:18.0` / `firestream-postgresql:17` so the side-loaded images are used |

## Quickstart

### 0. Prerequisites
`nix` (flakes enabled), `docker`, `kubectl`, `helm`, and **one** of:
- a running **host k3s** where you can talk to containerd without sudo — i.e.
  you're in the `k3s` group (`id | grep k3s`); or
- a **k3d** cluster — create one with `k3d cluster create odoo` and export
  `K3D_CLUSTER=odoo` (the deploy script imports images via `k3d image import`).

Everything else is provided by the dev shell — run `nix develop` (or
`direnv allow`).

### 1. (optional) Edit `config.nix`
Set the namespace, passwords, and vendored addons you want. The defaults work
out of the box.

### 2. Build + inspect the chart (no cluster needed)
```bash
make chart
grep -E 'firestream-odoo|storageClass|type:|odooPassword' result/values.yaml
```
Confirm the image repository is `firestream-odoo` (NOT repointed at a registry),
`storageClass: local-path`, and `service.type: ClusterIP`.

### 3. Deploy
```bash
make deploy        # builds images, side-loads them, helm upgrade --install
```

### 4. Reach Odoo
```bash
make status                       # wait until both pods are 1/1 Running
make port-forward                 # localhost:8069 -> svc/odoo
```
Open <http://127.0.0.1:8069> and log in with `user@example.com` /
`admin1234` (the `odooEmail` / `odooPassword` from `config.nix`).

### 5. Clean reinstall
```bash
make destroy       # deletes the namespace: release + PVCs
make deploy
```

## Vendoring extra addons (custom image)

Just like the GKE example, this builds a **custom Odoo image** by injecting
preferences into Firestream's Nix system — *inject preferences → get a custom
image, no fork.* Add repos to `vendoredAddons` in
[`config.nix`](./config.nix); they're fetched at build time and baked into
`/opt/odoo/vendor-addons` (wired into `addons_path`). The resulting image is
still named `firestream-odoo:18.0`, so the (un-repointed) chart picks it up once
`make deploy` side-loads it. Install a vendored module like any other (Apps →
Update Apps List). See [`../odoo-gke/README.md`](../odoo-gke/README.md) for the
`hash` workflow and using non-GitHub sources.

## Notes & gotchas

- **Separate image stores.** A host k3s (or k3d node) uses its own containerd
  store, independent of the Docker daemon. Rebuilding an image and `docker
  load`-ing it does **not** change what the cluster runs until it's re-imported
  — which `make deploy` does every run, so just re-run it after a code change.
- **First build is slow.** The Odoo image is ~2.25 GB and the github flake input
  is fetched once; subsequent builds are cached.
- **`firestream helm deploy odoo`.** The standalone Firestream CLI can also
  deploy this app from a published bundle, but the CLI does not side-load images
  — `make deploy` does, which is why this example uses the script. If you deploy
  via the CLI, run the side-load step from `scripts/deploy-local.sh` first.
- **In-repo testing (before this lands on `main`).** The flake pins the public
  github input, which only has the consumer API once merged. To test against
  your local checkout meanwhile:
  ```bash
  NIX_OVERRIDE="--override-input firestream path:../.." make deploy
  ```
