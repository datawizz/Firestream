# Next.js on GKE — a Firestream deployment (custom app)

A production pattern for running a **net-new Next.js app** as a Firestream
supported app on **GKE Autopilot**, with **your app baked into the image**,
images in **Artifact Registry**, the database credential in **Secret Manager**,
and a **GCE L7 ingress**. Sibling of [`../nextjs-k3s`](../nextjs-k3s) and
[`../nextjs-docker-compose`](../nextjs-docker-compose). See
[`../README.md`](../README.md) for the one-flake pattern.

## Layout

```
nextjs-gke/
├── flake.nix                       # imports firestream; packages.{nextjs-chart,nextjs-image,postgresql-image}
├── config.nix                      # ← YOU EDIT THIS: project, region, cluster, AR, domain, storage, secret name
├── app/                            # ← YOUR Next.js app, baked into the image
├── firestream/
│   ├── nextjs-overrides.nix        # CHART override: AR image refs, existingSecret, GCE ingress, persistence
│   └── nextjs-image-overrides.nix  # config.nextjs.vendoredApp = { src = ./app; npmDepsHash = ...; }
├── pulumi/                         # GKE Autopilot + Artifact Registry + Secret Manager + IAM
│   ├── __main__.py
│   ├── Pulumi.yaml
│   ├── Pulumi.dev.example.yaml      # copy → Pulumi.dev.yaml; set --secret values
│   └── requirements.txt
├── cloudbuild.yaml                 # build+push images → sync Secret → helm deploy
├── scripts/
│   ├── deploy-local.sh             # local equivalent of cloudbuild.yaml
│   └── sync-secrets.sh             # Secret Manager → in-cluster Secret
└── Makefile                        # chart / render / pulumi-* / images / deploy / destroy / clean
```

## How it works

1. **`pulumi up`** provisions a GKE Autopilot cluster, an Artifact Registry repo,
   the Secret Manager secrets (`nextjs-db-password`, `nextjs-db-postgres-password`),
   and a deployer service account. Copy the stack outputs into `config.nix`.
2. **Cloud Build** (or `make deploy` locally) builds the custom
   `firestream-nextjs` image from [`app/`](./app) + `firestream-postgresql`,
   pushes them to Artifact Registry, materialises the `nextjs-db-credentials`
   Kubernetes Secret from Secret Manager, and `helm upgrade --install`s the
   chart.
3. `firestream/nextjs-overrides.nix` **repoints** both image refs at Artifact
   Registry and sets `postgresql.auth.existingSecret`, so no passwords land in
   git or `values.yaml`. The chart wires `PGPASSWORD` from that Secret.

## Quickstart

### 0. Prerequisites
`nix` (flakes), `docker`, `gcloud` (authenticated), `kubectl`, `helm`, and
`pulumi`. `nix develop` (or `direnv allow`) provides the cloud CLIs.

### 1. Provision infrastructure
```bash
cd pulumi
cp Pulumi.dev.example.yaml Pulumi.dev.yaml
pulumi stack init dev
# set gcp:project / gcp:region / clusterName / arRepo and the --secret values
# (see the comments in Pulumi.dev.example.yaml), then:
pulumi up
```

### 2. Wire `config.nix`
Copy the Pulumi outputs (`clusterName`, `region`, `arHost`, `arRepo`) into
[`config.nix`](./config.nix), and set `domain` to your hostname.

### 3. Build the chart (no cluster needed)
```bash
make chart
grep -E "$(nix eval --raw --impure --expr '(import ./config.nix).arRepo')|ingressClassName|existingSecret" result/values.yaml
```
Confirm the image refs point at Artifact Registry and the ingress class is `gce`.

### 4. Deploy
```bash
make deploy        # build + push images, sync Secret, helm upgrade --install
kubectl -n nextjs get pods -w
```
Point DNS for `config.nix`'s `domain` at the ingress IP
(`kubectl -n nextjs get ingress`), then browse to it — the homepage renders the
live `select version(), now()` from firestream-postgresql.

### 5. Tear down
```bash
make destroy       # pulumi destroy (cluster, AR, secrets, IAM)
```

## Notes & gotchas

- **Autopilot ingress.** A `ClusterIP` service + GCE ingress yields a
  container-native (NEG) L7 LB. Provisioning the public IP + cert can take a few
  minutes after first deploy.
- **Immutable tags.** `imageTag = "latest"` is convenient but use a git SHA in
  real pipelines so rollouts are reproducible.
- **Make it your own:** edit anything under [`app/`](./app). If you change
  dependencies, update `npmDepsHash` in `firestream/nextjs-image-overrides.nix`.
