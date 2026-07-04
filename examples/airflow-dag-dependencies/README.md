# Airflow guest-DAG dependencies on local k3s / k3d — a Firestream deployment

A complete, copyable pattern for running the Firestream **Airflow** app on a
**local Kubernetes cluster** (host k3s or a throwaway k3d), where the headline
customization is **guest-DAG Python dependencies packaged as their own uv2nix
workspace**, resolved into a **separate, isolated venv** baked into the image via
`config.airflow.dagWorkspace`. Highlights:

- a self-contained guest workspace ([`dags-workspace/`](./dags-workspace)) —
  `pyproject.toml` + committed `uv.lock` — resolved via **uv2nix** into a
  SEPARATE venv baked at `/opt/firestream/airflow/dags-venv`, **isolated** from
  Airflow's own venv at `/opt/firestream/airflow/venv`;
- two env vars baked into every Airflow pod so DAGs can reach that venv:
  `FIRESTREAM_DAGS_VENV` and `FIRESTREAM_DAGS_PYTHON`;
- an example DAG ([`dags/guest_dependencies.py`](./dags/guest_dependencies.py))
  showing **both** consumption shapes (BashOperator → console-script, and
  `@task.external_python`), baked in via `config.airflow.vendoredDags`;
- the bundled **PostgreSQL** (metadata DB) and **Redis** (Celery broker)
  subcharts; **CeleryExecutor**;
- **inline credentials** from `config.nix`, **`local-path`** persistence,
  **ClusterIP + port-forward** access, `firestream-*` images **side-loaded** into
  containerd (no registry).

Local-dev sibling of [`../airflow-k3s`](../airflow-k3s) (which bakes a DAG via
`config.airflow.vendoredDags`). See [`../README.md`](../README.md) for the
general pattern.

## What this example demonstrates

`config.airflow.dagWorkspace` is the seam for *guest-DAG dependencies*. Real
DAGs need third-party Python packages (an API client, a math library, …), but
you must NOT drop those into Airflow's own environment — that risks version
conflicts with Airflow's tightly-pinned closure. Firestream's answer:

- **Declare** them as your own uv2nix workspace (`dags-workspace/`).
- **Resolve** them reproducibly at build time into a **separate** venv baked at
  `/opt/firestream/airflow/dags-venv` — Airflow's own venv is untouched.
- **Reach** them across a process boundary from your DAG via the two baked env
  vars.

### The isolation guarantee

The workspace's only dependency is `cowsay` — pure-Python, wheel-available, and
**not** in Airflow's closure. It imports from the guest venv but is absent from
Airflow's base venv. That is the proof the two venvs are isolated.

### The two consumption shapes (see `dags/guest_dependencies.py`)

1. **BashOperator → console-script (recommended).** Run
   `$FIRESTREAM_DAGS_VENV/bin/airflow-guest-dag-demo` in a separate process.
   The guest venv needs nothing from Airflow. *XCom caveat:* BashOperator pushes
   only the **last** stdout line to XCom, so the script emits one final line.
2. **`@task.external_python(python=os.environ["FIRESTREAM_DAGS_PYTHON"])`.** The
   callable body runs under the guest interpreter. Two honest tiers:
   - **Tier A** (this fixture): guest ships NO apache-airflow → cheap, but the
     callable gets serializable args in / a serializable value out and has **no**
     live Airflow context.
   - **Tier B**: pin `apache-airflow==3.0.3` in the guest to regain full context
     + in-callable XCom, at the cost of re-coupling to Airflow's full closure and
     a much larger image. A deliberate tradeoff — not free.

### The `uv.lock` freshness contract

uv2nix builds from the **committed `uv.lock`**. After editing
`dags-workspace/pyproject.toml`, regenerate and gate:

```bash
cd dags-workspace && uv lock && uv lock --check   # or: make lock-check
```

`requires-python` **must admit 3.12** (`>=3.12,<3.13`) — Airflow's interpreter
is `python312`, and Firestream's factory throws a legible build-time error if it
doesn't.

## Layout

```
airflow-dag-dependencies/
├── flake.nix                       # imports firestream; packages.{airflow-chart,airflow-image,postgresql-image,redis-image}
├── config.nix                      # ← YOU EDIT THIS: namespace, storage, passwords, crypto keys
├── dags-workspace/                 # ← the guest uv2nix workspace (deps baked into a SEPARATE venv)
│   ├── pyproject.toml              #   [project] + [project.scripts] + deps (cowsay)
│   ├── uv.lock                     #   committed lockfile (REQUIRED by uv2nix)
│   ├── README.md                   #   freshness contract + isolation notes
│   └── src/airflow_guest_dags/     #   airflow-guest-dag-demo -> main()
│       ├── __init__.py
│       └── cli.py
├── dags/
│   └── guest_dependencies.py       # ← example DAG (BashOperator + external_python shapes)
├── firestream/
│   ├── airflow-overrides.nix       # sparse CHART override (CeleryExecutor, local-path, inline creds)
│   └── airflow-image-overrides.nix # config.airflow.dagWorkspace = { src = ../dags-workspace; }  (+ vendoredDags for the DAG file)
├── scripts/
│   └── deploy-local.sh             # uv lock --check → build → side-load 3 images → helm upgrade --install
└── Makefile                        # chart / render / lock-check / deploy / port-forward / status / destroy / clean
```

## How the guest venv gets baked in

`firestream/airflow-image-overrides.nix` sets

```nix
config.airflow.dagWorkspace = { src = ../dags-workspace; };
```

The Airflow container's uv2nix factory resolves `../dags-workspace` (from its
committed `uv.lock`) into a venv baked at `/opt/firestream/airflow/dags-venv`,
distinct from Airflow's own venv, and bakes `FIRESTREAM_DAGS_VENV` /
`FIRESTREAM_DAGS_PYTHON`. The same override also vendors `../dags` (the DAG file
itself) into `/opt/firestream/airflow/dags` via `config.airflow.vendoredDags` —
the two are orthogonal: the DAG file is what the scheduler scans; its deps live
in the separate guest venv. Because the chart override does **not** repoint the
image, the cluster runs this custom `firestream-airflow:3.0.3` once `make deploy`
side-loads it. *Inject preferences → custom image, no fork.*

## Quickstart

### 0. Prerequisites
`nix` (flakes), `docker`, `kubectl`, `helm`, `uv`, and **one** of:
- a running **host k3s** where you can talk to containerd without sudo (you're
  in the `k3s` group — `id | grep k3s`); or
- a **k3d** cluster (`k3d cluster create airflow`, then `export K3D_CLUSTER=airflow`).

Everything else is in the dev shell — `nix develop` (or `direnv allow`).

### 1. (optional) Edit the workspace or `config.nix`
Add a dependency in `dags-workspace/pyproject.toml`, then `make lock-check` (or
`cd dags-workspace && uv lock && uv lock --check`). Set the namespace, passwords,
and crypto keys in `config.nix` — generate a real Fernet key with
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
make deploy        # uv lock --check, builds 3 images, side-loads them, helm upgrade --install
make status        # wait until all pods are Ready (Celery → several pods)
```

### 4. Reach Airflow + run the DAG
```bash
make port-forward  # localhost:8080 -> svc/airflow-web
```
Open <http://127.0.0.1:8080>, log in `admin` / `admin1234`, and confirm
**`guest_dependencies`** is in the DAG list. Unpause + trigger it:
- `run_guest_cli` runs the guest venv's console-script (BashOperator);
- `run_guest_python` imports `cowsay` under the guest interpreter
  (`@task.external_python`).

### 5. Clean reinstall
```bash
make destroy && make deploy
```

## Notes & gotchas

- **Isolation, not sharing.** The guest venv is deliberately NOT on Airflow's
  `PYTHONPATH`. DAG *definition* code runs under Airflow's interpreter; guest
  deps are reached only across a process boundary (BashOperator or
  external_python). Importing `cowsay` at the top level of the DAG file would
  fail — that's the feature working as intended.
- **Stale lock = wrong build.** uv2nix builds from the committed `uv.lock`;
  `make deploy` runs `uv lock --check` first and fails fast if it's stale.
- **Celery footprint.** Several pods (api-server, scheduler, worker, triggerer,
  dag-processor, postgresql, redis). For a tiny machine, switch
  `executor = "LocalExecutor"` in the override (drops redis + worker).
- **Separate image stores.** Host k3s / k3d uses its own containerd, independent
  of Docker. `make deploy` re-imports every run, so re-run after editing the
  workspace or a DAG.
- **First build is slow** (large Airflow image + one-time github fetch + guest
  venv resolution); cached after.
- **In-repo testing before merge** (the flake pins github main):
  ```bash
  NIX_OVERRIDE="--override-input firestream path:../.." make deploy
  ```
