# Odoo on GKE — a Firestream deployment

This is a complete, copyable pattern for running the Firestream **Odoo** app on
**Google Kubernetes Engine**, with:

- **Pulumi (Python)** provisioning a GKE Autopilot cluster, an Artifact Registry
  repo, GCP Secret Manager secrets, and the IAM Cloud Build needs;
- **GCP Secret Manager** holding the admin + database passwords, materialised
  into Kubernetes Secrets at deploy time (the chart references them by name via
  `existingSecret` — no passwords ever land in git or in `values.yaml`);
- **Cloud Build** building and pushing the `firestream-*` images, building your
  customized chart bundle, and deploying it.

The whole thing is anchored by one `flake.nix` that imports Firestream and a
single sparse override module — see [`../README.md`](../README.md) for the
general pattern.

## Layout

```
odoo-gke/
├── flake.nix                  # imports firestream; packages.odoo-chart = charts.odoo.eval(...).chartBundle
├── config.nix                 # ← YOU EDIT THIS: project, region, cluster, AR, domain, storage, secret names
├── firestream/
│   └── odoo-overrides.nix     # the sparse "make it your own" override (reads config.nix)
├── pulumi/                    # GKE + Artifact Registry + Secret Manager + IAM (Python)
├── cloudbuild.yaml            # build images → build chart → auth GKE → sync secrets → deploy
├── scripts/
│   ├── sync-secrets.sh        # GCP Secret Manager → Kubernetes Secrets (idempotent)
│   └── deploy-local.sh        # the full pipeline, locally, for the dev loop
└── Makefile                   # chart / render / pulumi-* / deploy / destroy
```

## What gets overridden

`firestream/odoo-overrides.nix` sets only deployment-specific values on top of
the Firestream defaults:

| Override | Why |
|----------|-----|
| `existingSecret`, `postgresql.auth.existingSecret` | Read credentials from K8s Secrets (sourced from Secret Manager) instead of inline/random passwords |
| `image.*`, `postgresql.image.*` | Pull the `firestream-*` images from your Artifact Registry |
| `persistence.*`, `postgresql.primary.persistence.*` | Bind the Odoo filestore + DB to a GKE StorageClass |
| `ingress.*` (class `gce`) | Expose Odoo through a GKE L7 load balancer |

Credentials map to these Secret keys (Bitnami conventions):

- `odoo-credentials` → key `odoo-password`
- `odoo-db-credentials` → keys `password` (app user) + `postgres-password` (admin)

## Quickstart

### 0. Prerequisites
`gcloud` (authenticated: `gcloud auth login` + `gcloud auth application-default
login`), `nix` (flakes enabled), and `docker`. Everything else is provided by
the dev shell — run `nix develop` (or `direnv allow`).

### 1. Provision infrastructure with Pulumi
```bash
cd pulumi
python -m venv venv && . venv/bin/activate && pip install -r requirements.txt
pulumi stack init dev
pulumi config set gcp:project   YOUR_PROJECT
pulumi config set gcp:region    us-central1
pulumi config set clusterName   firestream-odoo
pulumi config set arRepo        firestream
pulumi config set --secret odooPassword        "$(openssl rand -base64 24)"
pulumi config set --secret dbPassword          "$(openssl rand -base64 24)"
pulumi config set --secret dbPostgresPassword  "$(openssl rand -base64 24)"
pulumi up
```

### 2. Wire `config.nix`
Copy the Pulumi stack outputs into [`config.nix`](./config.nix):
```bash
pulumi stack output       # clusterName, region, arHost, arRepo
```
Set `projectId`, `region`, `clusterName`, `arHost`, `arRepo`, `domain`, and
`storageClass` to match.

### 3. Build + inspect the chart (no cluster needed)
```bash
make chart      # nix build .#odoo-chart
make render     # print the rendered manifests
grep -E 'existingSecret|registry|storageClass' result/values.yaml
```
Confirm `existingSecret: odoo-credentials`, the image `registry` points at your
Artifact Registry host, and the storage class is set.

### 4. Deploy
Locally:
```bash
make deploy     # scripts/deploy-local.sh: push images, sync secrets, helm deploy
```
Or via Cloud Build (connect `cloudbuild.yaml` to a trigger, with substitutions
matching `config.nix`):
```bash
gcloud builds submit --config cloudbuild.yaml \
  --substitutions=_REGION=us-central1,_CLUSTER=firestream-odoo,_NAMESPACE=odoo,_AR_HOST=us-central1-docker.pkg.dev,_AR_REPO=YOUR_PROJECT/firestream,_IMAGE_TAG=latest
```

### 5. Reach Odoo
```bash
kubectl -n odoo get pods -w
kubectl -n odoo get ingress      # note the LB address; point your DNS at it
```

## Vendoring extra addons (custom image)

This example doesn't just customize the Helm chart — it builds a **custom Odoo
image** by injecting preferences into Firestream's Nix system. That's the core
Firestream value prop: *inject preferences → get a custom image, no fork.*

Two parallel override seams:

| Seam | File | Customizes |
|------|------|------------|
| `charts.odoo.eval` | `firestream/odoo-overrides.nix` | Helm **chart** values (image refs, ingress, persistence, secrets) |
| `images.odoo.eval` | `firestream/odoo-image-overrides.nix` | The **container image** itself (vendored addons, env, paths, …) |

To bake third-party Odoo addon repositories into the image at build time, add
them to `vendoredAddons` in [`config.nix`](./config.nix):

```nix
vendoredAddons = [
  { name = "oca-server-tools";
    owner = "OCA"; repo = "server-tools";
    rev  = "<commit-sha>";                 # pin a commit, not a branch
    hash = "sha256-…";                     # fetchFromGitHub SRI hash
    modules = [ "base_fontawesome" ]; }    # omit to vendor the whole repo
];
```

`firestream/odoo-image-overrides.nix` feeds that list to
`firestream.lib.<sys>.images.odoo.eval`, which fetches each repo, lays its modules
into a baked, read-only `/opt/odoo/vendor-addons`, and wires it into `addons_path`.
`nix build .#odoo-image` then produces the image with those modules baked in;
because Cloud Build pushes `.#odoo-image` as `<AR>/firestream-odoo:<tag>` and
`odoo-overrides.nix` points the chart at that same ref, the customized image is
what the cluster runs. Install a vendored module like any other (Apps → Update
Apps List, or `-i <module>`).

**Getting the `hash`:** start from `lib.fakeHash` (or any wrong value), run
`nix build .#odoo-image`, and copy the `got: sha256-…` from the error. Use any
non-GitHub source by setting `src = <derivation>` instead of
`owner/repo/rev/hash`.

## Notes & follow-ups

- **Image-registry override**: the chart's default image slots are injected by
  Firestream (`docker.io/firestream-*`). The override repoints them at Artifact
  Registry; per `bin/nix/firestream/charts/lib/inject-container-images.nix`, an
  explicit user value wins over the injected one. Verify in `result/values.yaml`
  after `make chart` (the rendered pod specs pull from your Artifact Registry).
  Note: `chart-manifest.json`'s `images` catalogue still lists the Firestream
  default tags — it is sourced from the image-injection metadata and is only
  used by the `firestream` CLI's image-preload step; the deployed images come
  from `values.yaml`, which this example deploys via `bin/deploy`.
- **TLS / managed certs / DNS**: not wired here. Add a `ManagedCertificate` +
  `networking.gke.io/managed-certificates` ingress annotation and an A record
  for `config.nix:domain` as a follow-up.
- **SMTP**: optional. Uncomment the `smtp*` block in `odoo-overrides.nix`, add a
  `smtp-password` secret, and extend `sync-secrets.sh`.
- **Cloud SQL**: this example uses the bundled in-cluster Postgres subchart. To
  use Cloud SQL instead, set `postgresql.enabled = false` and wire
  `externalDatabase.*` (which also supports `existingSecret`).
- **Trigger wiring**: connecting `cloudbuild.yaml` to your repo is a one-time
  console step; Pulumi already grants the Cloud Build SA the needed roles.
