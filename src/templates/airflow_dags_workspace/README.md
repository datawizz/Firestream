# Example guest-DAG uv2nix workspace

This is a copy-and-adapt fixture for Firestream Airflow's
`options.airflow.dagWorkspace` seam. It declares the Python dependencies your
guest DAGs need as **your own** uv2nix workspace, which Firestream resolves and
bakes into a **separate** venv at build time — reproducible, isolated from
Airflow's own closure, and first-class in downstream Nix builds of the image.

## Why it lives here (not under the airflow container)

`src/containers/firestream/airflow/` **is** the Airflow uv2nix `workspaceRoot`.
A nested `pyproject.toml` inside it would confuse `loadWorkspace` and pollute the
Airflow image source, so this fixture deliberately lives under `src/templates/`
(alongside `standard_project/`), which nothing scans as a container or workspace.

## Layout

```
airflow_dags_workspace/
├── pyproject.toml          # [project] + [project.scripts] console-script + deps
├── uv.lock                 # committed lockfile (REQUIRED by uv2nix)
├── src/firestream_dags_example/
│   ├── __init__.py
│   └── cli.py              # firestream-dag-demo -> main()
└── dags/
    └── example_dag_workspace.py   # BashOperator + external_python shapes
```

An `overrides.nix` is intentionally omitted: the sole dependency (`cowsay`) is
pure-Python and wheel-available, so no C-extension build fixups are needed. Add
a `{ pkgs, lib }: { wheelOverrides = ...; sourceOverrides = ...; }` file here
only if one of your deps needs system libraries at build/runtime — Firestream
auto-loads `overrides.nix` from the workspace dir if present.

## The isolation marker

The single dependency is `cowsay`: pure-Python, wheel-available, and **not**
part of Airflow's own dependency closure (verify with
`grep -c cowsay src/containers/firestream/airflow/uv.lock` → `0`). Importing it
from the guest venv but not from Airflow's base venv is what proves the two
venvs are isolated.

## Freshness contract (user responsibility)

uv2nix builds from the **committed `uv.lock`**. A stale lock produces a wrong or
failed build. After editing `pyproject.toml`, regenerate and gate on:

```bash
uv lock          # regenerate
uv lock --check  # freshness gate: fails if pyproject.toml and uv.lock disagree
```

`requires-python` **must admit 3.12** (`>=3.12,<3.13` here) — Airflow's
interpreter is `python312`, and Firestream's factory throws a legible error if
the guest workspace excludes it.

## Wiring it up

In your Airflow container options:

```nix
options.airflow.dagWorkspace = {
  src = ./path/to/this/workspace;   # dir with pyproject.toml + uv.lock
  # overrides = ./overrides.nix;    # optional; auto-loaded from src if present
};
```

Firestream then bakes:

- the guest venv at `/opt/firestream/airflow/dags-venv`
- `FIRESTREAM_DAGS_VENV=/opt/firestream/airflow/dags-venv`
- `FIRESTREAM_DAGS_PYTHON=/opt/firestream/airflow/dags-venv/bin/python`

into every Airflow pod. See `dags/example_dag_workspace.py` for the two
consumption shapes (BashOperator → console-script, and `@task.external_python`).
