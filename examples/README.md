# Firestream Deployment Examples

This directory holds **reference deployments** — self-contained patterns for
taking a Firestream-supported app, layering a small override to "make it your
own," and shipping it to a real environment.

Each example is a standalone git-repo pattern you can copy out wholesale: a root
`flake.nix` that imports Firestream as an input, plus whatever cloud plumbing
(IaC, CI) that target needs.

## The pattern

A "Firestream deployment" is, at its core, **one flake**:

```nix
# flake.nix
inputs.firestream.url = "github:Cogent-Creation-Co/Firestream-one-flake-to-rule-them-all";

# ...then re-evaluate a supported app with YOUR overrides deep-merged on top:
packages.odoo-chart =
  (firestream.lib.${system}.charts.odoo.eval (import ./firestream/odoo-overrides.nix))
    .chartBundle;
```

`charts.<app>.eval` is the standardized customization seam
(`docs/firestream-supported-app.md` §7). It re-runs the chart's typed-option
evaluation with your module added last, so you override a single value without
re-stating the whole binding. It returns a `chartBundle` derivation containing
the forked chart, a Nix-emitted `values.yaml`, a `chart-manifest.json`, and an
executable `bin/deploy` (`helm upgrade --install …`).

Your override module is a **strict subset** of the chart's value surface (the
same keys as `src/charts/firestream/<app>/values.yaml`). You never fork chart
templates and never restate Firestream's image wiring — you only set what's
specific to you (credentials, storage class, ingress hostname, image registry).

The result is deployable three ways, all equivalent:

- `nix run .#deploy -- --namespace <ns>` (or `./result/bin/deploy …`)
- `helm upgrade --install <rel> result/chart -f result/values.yaml`
- `firestream helm deploy <app>` (when the bundle is published into a
  `FIRESTREAM_CHARTS_DIR` aggregate with an `index.json`)

## Examples

Each app ships as a trio — the *same app* via the *same* `charts.<app>.eval` /
`images.<app>.eval` seams, aimed at three runtimes (production cloud, a local
cluster, plain Docker). The `*-gke` variant is the production shape; the `*-k3s`
and `*-docker-compose` variants are its local-dev shapes.

**Odoo** — customization: vendored third-party addons baked into the image.

| Example | Target | Highlights |
|---------|--------|------------|
| [`odoo-gke/`](./odoo-gke) | GKE Autopilot | Pulumi cluster + Artifact Registry + Secret Manager; Cloud Build deploy; credentials via `existingSecret` |
| [`odoo-k3s/`](./odoo-k3s) | Local k3s / k3d | Build → side-load images into containerd → helm → port-forward; inline credentials; `local-path` storage |
| [`odoo-docker-compose/`](./odoo-docker-compose) | Docker Compose | No Kubernetes; reuses Firestream's generated compose stack; custom image with vendored addons |

**Airflow** — customization: a **custom DAG baked into the image** (the Airflow
analogue of odoo's vendored addons, via `config.airflow.vendoredDags`).
CeleryExecutor + bundled PostgreSQL & Redis.

| Example | Target | Highlights |
|---------|--------|------------|
| [`airflow-gke/`](./airflow-gke) | GKE Autopilot | Baked DAG; 3 images → Artifact Registry + Secret Manager; Cloud Build; `existingSecret` |
| [`airflow-k3s/`](./airflow-k3s) | Local k3s / k3d | Baked DAG; side-load 3 images into containerd → helm → port-forward; inline credentials; `local-path` |
| [`airflow-docker-compose/`](./airflow-docker-compose) | Docker Compose | Baked DAG; reuses Firestream's generated multi-service compose stack |

**Next.js** — a **net-new (non-Bitnami) supported app**: a production Next.js
server built to standalone output by the Firestream Node factory
(`mkNodePackage` + `mkNodeContainerModule`), with your app baked into the image
via `config.nextjs.vendoredApp` and a **bundled PostgreSQL** subchart running
the firestream-postgresql image. The homepage runs a live `SELECT` against the
database.

| Example | Target | Highlights |
|---------|--------|------------|
| [`nextjs-gke/`](./nextjs-gke) | GKE Autopilot | Custom app; 2 images → Artifact Registry + Secret Manager; Cloud Build; GCE ingress; `existingSecret` |
| [`nextjs-k3s/`](./nextjs-k3s) | Local k3s / k3d | Custom app; side-load 2 images into containerd → helm → port-forward; inline credentials; `local-path` |
| [`nextjs-docker-compose/`](./nextjs-docker-compose) | Docker Compose | Custom app; reuses Firestream's generated nextjs + postgresql compose stack |

Future examples follow the same shape — a root flake + a sparse override + the
target's plumbing. Start a new one by copying the closest existing example and
swapping the app name in the `charts.<app>.eval` call.
