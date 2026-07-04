# Guest-DAG uv2nix workspace (`airflow-guest-dags`)

This is the **self-contained** Python workspace whose dependencies your guest
DAGs need. Firestream's `config.airflow.dagWorkspace` seam resolves it via
uv2nix and bakes it into a **separate** venv at
`/opt/firestream/airflow/dags-venv` inside the Airflow image — reproducible,
isolated from Airflow's own closure, and first-class in downstream Nix builds.

It is embedded here (rather than referenced from `src/templates/`) so this
example is copyable wholesale.

## Layout

```
dags-workspace/
├── pyproject.toml          # [project] + [project.scripts] console-script + deps
├── uv.lock                 # committed lockfile (REQUIRED by uv2nix)
└── src/airflow_guest_dags/
    ├── __init__.py
    └── cli.py              # airflow-guest-dag-demo -> main()
```

The DAG that consumes this workspace lives in `../dags/guest_dependencies.py`
(next to the other example files), so the scheduler scans it as a normal DAG.

An `overrides.nix` is intentionally omitted: the sole dependency (`cowsay`) is
pure-Python and wheel-available, so no C-extension build fixups are needed. Add
a `{ pkgs, lib }: { wheelOverrides = ...; sourceOverrides = ...; }` file here
only if one of your deps needs system libraries at build/runtime — Firestream
auto-loads `overrides.nix` from the workspace dir if present (wire it via the
`overrides` field of `config.airflow.dagWorkspace`).

## The isolation marker

The single dependency is `cowsay`: pure-Python, wheel-available, and **not**
part of Airflow's own dependency closure. Importing it from the guest venv but
not from Airflow's base venv is what proves the two venvs are isolated.

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
