# PRD: Object-Storage Filestore Backends for Odoo (GCS + S3) via OCA, Nix-Packaged

**Status:** Draft v1
**Owner:** Center Point Consulting Group
**Scope:** Open-source Odoo (Community) + OCA `storage` addons, with all system and Python dependencies provided declaratively by Nix.

---

## 1. Summary

Odoo stores binary attachments (`ir.attachment` records — uploaded documents, generated PDFs, web assets, product images) on the local filesystem by default, with only metadata in PostgreSQL. The filestore directory is a stateful, node-local volume. This is the single largest obstacle to running Odoo as a horizontally-scalable, reproducible, cattle-not-pets workload: any second app server, any ephemeral container, any blue/green deploy must somehow share that directory.

This PRD specifies a backend that relocates the Odoo filestore to object storage — **Google Cloud Storage (GCS)** and **Amazon S3** (and, for free, any other `fsspec`-supported protocol) — using only open-source Odoo and OCA modules, with every underlying package (Odoo, Python, `fsspec`, `s3fs`, `gcsfs`) sourced from Nix rather than `pip`. The deliverable is a packaged, declaratively-configured Odoo deployment in which attachments live in a bucket and app servers are stateless.

## 2. Goals

1. Attachments persist to GCS or S3 instead of the local filestore, transparently to all Odoo business modules.
2. App servers hold no durable attachment state, enabling N-replica horizontal scaling and immutable deploys.
3. Backend selection and credentials are fully declarative — defined in `odoo.conf` / Nix, not clicked into the UI.
4. All packages reproducible through Nix; no `pip install` at runtime, no impure network fetches in the build.
5. Native, first-class GCS support (not an S3-compatibility shim) alongside native S3.
6. Multi-tenant friendly: one configuration template serves many databases, each isolated to its own bucket path.
7. A safe, reversible migration path from an existing on-disk filestore.

## 3. Non-Goals

- **DMS / document-management UX.** This is plumbing for `ir.attachment`, not a folder-browser, versioning UI, or Odoo Documents replacement.
- **CDN / public asset serving.** Out of scope beyond the optional nginx x-sendfile hand-off (§9.3).
- **PostgreSQL or `filestore`-as-DB strategies.** Storing attachments in the DB (`ir_attachment.location = db`) is explicitly rejected as a primary store for scale reasons; it survives only as a narrow optimization for tiny hot assets (§8).
- **Enterprise Odoo features.** Community edition only.
- **Paid Apps-Store modules** (Webkul, cs_gcs_document_management, etc.). AGPL OCA only, to keep the dependency closure controllable and license-clean.

## 4. Background & Prior Art

There are two established lineages of community prior art. We select the modern one and document the legacy one as a fallback.

### 4.1 Modern path (selected): OCA `fs_storage` + `fs_attachment`

The OCA `storage` repository was refactored (RFC #251) to rebase its backends on Odoo's standard `ir.attachment` model instead of a bespoke `storage.file` model. The result is a clean two-layer design with active `18.0` branches:

- **`fs_storage`** — a technical addon that exposes any storage location as an `fsspec.AbstractFileSystem`. The set of usable protocols is exactly the set of installed `fsspec` implementations. Backends can be defined entirely from the config file.
- **`fs_attachment`** — extends Odoo's attachment read/write/delete hooks to route file content into any `fs_storage`-defined filesystem. Adds meaningful filenames (vs. Odoo's content-checksum names), an autovacuum GC to delete unreferenced objects, end-to-end streaming, and optional x-sendfile serving.

This is the recommended foundation because GCS is a native `fsspec` protocol via `gcsfs` and S3 via `s3fs` — one module family, no S3-compat hacks, current Odoo branch.

### 4.2 Legacy path (fallback): Camptocamp `odoo-cloud-platform`

`attachment_s3`, `attachment_azure`, and `base_attachment_object_storage` from `camptocamp/odoo-cloud-platform`. These hook in the classic way: set the `ir_attachment.location` system parameter to an `s3://…` value and call `force_storage()` to migrate. Battle-tested over many years and production deployments, but S3-specific — GCS only via the S3-compatible XML endpoint + HMAC interoperability keys. Retained as a fallback for S3-only sites already standardized on this tooling.

### 4.3 Native Odoo mechanism

Odoo core exposes three `ir.attachment` hook methods (`_file_read`, `_file_write`, `_file_delete`) and the `ir_attachment.location` system parameter. Both lineages above plug into these hooks; no core patching is required.

## 5. Architecture

```
┌───────────────────────────────────────────────┐
│  Odoo business modules (sale, account, web…)   │
│  create/read ir.attachment as normal           │
└───────────────────────┬───────────────────────┘
                        │  Binary fields / attachments
┌───────────────────────▼───────────────────────┐
│  fs_attachment                                 │
│   - overrides _file_read/_write/_delete        │
│   - meaningful filenames, GC, streaming        │
└───────────────────────┬───────────────────────┘
                        │  selects storage by code
┌───────────────────────▼───────────────────────┐
│  fs_storage  (fs.storage records)              │
│   - wraps fsspec.AbstractFileSystem            │
│   - protocol + options + directory_path        │
└───────┬───────────────────────┬───────────────┘
        │ protocol=gcs           │ protocol=s3
┌───────▼────────┐      ┌────────▼────────┐
│  gcsfs         │      │  s3fs           │
│  → GCS bucket  │      │  → S3 bucket    │
└────────────────┘      └─────────────────┘
        ▲                        ▲
        └──────── Nix-provided Python deps ───────┘
```

The `store_fname` on each attachment is namespaced by backend code (`odoofs://<hash>`), so multiple backends can coexist and existing attachments keep resolving against the backend they were written to even after the default changes.

## 6. Dependencies (Nix-provided)

All of the following come from `nixpkgs`, not `pip`. The OCA addons themselves are not in `nixpkgs` and are packaged as fetched derivations (§7).

| Dependency | Source | Purpose |
|---|---|---|
| `odoo` (Community, target 18.x) | `pkgs.odoo` | Application runtime |
| `python3Packages.fsspec` | nixpkgs | Filesystem abstraction used by `fs_storage` |
| `python3Packages.s3fs` | nixpkgs | S3 protocol implementation |
| `python3Packages.gcsfs` | nixpkgs | GCS protocol implementation |
| `python3Packages.aiohttp` | nixpkgs (transitive) | Async HTTP for s3fs/gcsfs |
| `python3Packages.google-auth` | nixpkgs (transitive) | GCS service-account / Workload Identity auth |
| `python3Packages.botocore` / `aiobotocore` | nixpkgs (transitive) | S3 auth + signing |

**Key principle:** the OCA addons are *thin*. The real dependency surface is the `fsspec` + `s3fs`/`gcsfs` Python stack. Getting those into the Odoo Python environment correctly is the bulk of the packaging work; the addons are just copied into the addons path.

## 7. Nix Packaging

### 7.1 Strategy

1. Build a Python environment that is exactly Odoo's interpreter plus `fsspec`, `s3fs`, `gcsfs` and their closures.
2. Package the relevant subtree of `OCA/storage` (`fs_storage`, `fs_attachment`, and `fs_storage_*` protocol helpers if used) as a derivation that unpacks into a directory suitable for `addons_path`.
3. Wrap Odoo so its Python path includes the augmented environment and its `addons_path` includes the OCA derivation.

### 7.2 Sketch

```nix
{ pkgs ? import <nixpkgs> {} }:

let
  # 1. OCA storage addons, pinned by rev/hash (impurity-free)
  oca-storage = pkgs.fetchFromGitHub {
    owner = "OCA";
    repo  = "storage";
    rev   = "<pin-to-18.0-commit>";
    hash  = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  # 2. Only ship the addons we actually use
  storageAddons = pkgs.runCommand "oca-storage-addons" {} ''
    mkdir -p $out
    for m in fs_storage fs_attachment; do
      cp -r ${oca-storage}/$m $out/$m
    done
  '';

  # 3. Augment Odoo's Python with the fsspec stack
  odoo = pkgs.odoo.override (old: {
    # depending on the nixpkgs odoo expression, extend pythonPath /
    # propagatedBuildInputs with the fsspec backends:
    extraPythonPackages = ps: (old.extraPythonPackages or (_: [])) ps ++ [
      ps.fsspec
      ps.s3fs
      ps.gcsfs
    ];
  });
in
pkgs.writeShellApplication {
  name = "odoo-objstore";
  runtimeInputs = [ odoo ];
  text = ''
    exec odoo \
      --addons-path="${storageAddons}:${odoo}/share/odoo/addons" \
      --config="''${ODOO_RC:-/etc/odoo/odoo.conf}" \
      "$@"
  '';
}
```

> The exact override surface (`extraPythonPackages`, `withPackages`, or a `python.buildEnv` wrapper) depends on the `odoo` expression in the pinned nixpkgs. The invariant is: **Odoo's interpreter must import `fsspec`, `s3fs`, and `gcsfs` at runtime.** Validate with `odoo shell` → `import s3fs, gcsfs`.

### 7.3 NixOS module integration

Expose the wrapped package via a NixOS service that templates `odoo.conf` (§9) and mounts credentials. Secrets (SA JSON, S3 keys) are delivered via `age`/`sops-nix` or systemd `LoadCredential`, never baked into the store. The config references them by path or `$ENV_VAR` (see §9.2 env-var resolution).

## 8. Configuration

### 8.1 Declarative backend definition (the important part)

`fs_storage` reads backend definitions straight from `odoo.conf`, so no UI clicking is required and the whole thing stays in Nix-managed config. Each `[fs_storage.<code>]` section maps to an `fs.storage` record keyed by `<code>`.

**GCS (native, via gcsfs):**

```ini
[fs_storage.gcsprod]
protocol = gcs
options = {"project": "cpcg-prod", "token": "/run/credentials/odoo/gcs-sa.json"}
directory_path = cpcg-odoo-filestore
```

**S3 (native, via s3fs):**

```ini
[fs_storage.s3prod]
protocol = s3
options = {"key": "$AWS_ACCESS_KEY_ID", "secret": "$AWS_SECRET_ACCESS_KEY", "client_kwargs": {"region_name": "us-west-2"}}
directory_path = cpcg-odoo-filestore
```

**GCS via S3-compat (only if using the legacy `attachment_s3` fallback):**

```ini
options = {"key": "$GCS_HMAC_KEY", "secret": "$GCS_HMAC_SECRET", "client_kwargs": {"endpoint_url": "https://storage.googleapis.com"}}
```

### 8.2 Making it the default attachment store

The backend must be marked as the default for attachments (the `Use As Default For Attachment` flag on the `fs.storage` record, or the equivalent config field). Note the ordering caveat: attachments created *before* `fs_attachment` loads during a module update — e.g. some icons/assets — still land in the location named by `ir_attachment.location` (`file` by default). Plan for a small residue on local disk; it is harmless.

### 8.3 Hybrid: keep tiny hot assets in DB/disk

Object-store round-trips add latency that hurts for the many small images (128/256px kanban/list thumbnails) and JS/CSS assets Odoo reads constantly. The `base_attachment_object_storage` lineage exposes force-DB rules (a JSON map of mimetype → size threshold) so small images and all `application/javascript` + `text/css` stay in PostgreSQL while everything else goes to the bucket. This also improves DB portability (assets travel with a DB dump). Mirror this policy on the selected backend.

### 8.4 Multi-tenancy

`directory_path` supports `{db_name}` substitution, evaluated per database. A single templated config isolates every tenant to its own prefix:

```ini
[fs_storage.gcsprod]
protocol = gcs
options = {"project": "cpcg-prod", "token": "/run/credentials/odoo/gcs-sa.json"}
directory_path = cpcg-odoo-filestore/{db_name}
```

This is the key enabler for a config-file-only multi-tenant setup: no per-tenant records to provision.

## 9. Operational Concerns

### 9.1 Authentication

- **GCS:** prefer Workload Identity (GKE) or attached service-account (GCE) so no key material exists at all; fall back to SA JSON via `token` path delivered as a systemd credential. Bucket-level IAM: grant the SA `roles/storage.objectAdmin` scoped to the bucket.
- **S3:** prefer IRSA (EKS) / instance profile; fall back to access/secret keys via env vars resolved by the `$`-prefix option. IAM policy scoped to `s3:GetObject/PutObject/DeleteObject/ListBucket` on the bucket ARN.

### 9.2 Secrets & env-var resolution

`fs_storage` can resolve option values beginning with `$` from environment variables (the "resolve env vars" option). This keeps secrets out of the Nix store and out of the database: the config references `$AWS_SECRET_ACCESS_KEY`, the value arrives via systemd `EnvironmentFile`/`LoadCredential`, `sops-nix`, or `age`.

### 9.3 Serving performance

Default behavior streams content end-to-end through Odoo. For large/public files, enable x-sendfile so nginx proxies directly to the object's URL instead of pulling bytes through the Python workers. Combine with §8.3 force-DB rules so the hot small-asset path never touches the network.

### 9.4 Garbage collection

Enable `fs_attachment`'s autovacuum GC so deleting an attachment in Odoo eventually removes the object from the bucket — important because object stores bill on stored bytes. Validate GC behavior in staging before trusting it in prod; mis-scoped GC against a shared bucket is the highest-blast-radius failure mode here.

### 9.5 Transactions

`fs_attachment` honors Odoo's transactional semantics for attachment writes (rollback removes orphaned objects via GC). `fsspec`'s own transaction bridge is partial — it covers files written via the `open()` path, not arbitrary `rm`/`mv`. Don't assume cross-object atomicity.

## 10. Migration Plan

1. **Stage.** Deploy `fs_storage` + `fs_attachment`, define a *read-write staging* backend pointed at a staging bucket, leave production filestore as default. New attachments flow to the bucket; verify reads/writes/GC.
2. **Cutover (default switch).** Mark the production backend as default for attachments. From this point new attachments target the bucket; existing ones still resolve from disk via their `store_fname` backend code.
3. **Backfill.** Run the bulk migration (`force_storage()` in the legacy path; the equivalent storage migration in `fs_attachment`) to move pre-existing on-disk attachments into the bucket. Idempotent and resumable.
4. **Verify.** Spot-check attachment downloads across modules (invoices, sale orders, web assets); confirm checksum integrity; confirm GC of deleted records.
5. **Decommission.** Once backfill is verified, the local filestore volume can shrink to the small pre-load residue (§8.2). Keep it; do not delete the directory.

**Rollback:** because each attachment records its own backend in `store_fname`, reverting the default backend leaves already-migrated objects readable as long as credentials remain valid. Keep the bucket and SA alive through any rollback window.

## 11. Validation & Testing

- **Unit/import smoke:** `odoo shell` → `import fsspec, s3fs, gcsfs` succeeds in the Nix-built interpreter.
- **Backend round-trip:** create an attachment, assert the object appears in the bucket under the expected `{db_name}` prefix, read it back, checksum-match.
- **Default routing:** new attachments land in the bucket; force-DB rules keep thumbnails/assets in PG.
- **GC:** delete an attachment, run autovacuum, assert the object is gone.
- **Multi-tenant isolation:** two databases write to disjoint prefixes; neither can read the other's objects.
- **Failure injection:** revoke credentials mid-run; assert connection-check eviction and a clean error, not corruption.
- **Reproducibility:** `nix build` is hermetic; pinned `rev` + `hash` for `OCA/storage`; no impure fetch.

## 12. Milestones

| Phase | Deliverable |
|---|---|
| M1 | Nix derivation: Odoo + `fsspec`/`s3fs`/`gcsfs`; `import` smoke test green |
| M2 | OCA `storage` addons packaged into addons_path; modules install on a test DB |
| M3 | Declarative GCS backend via `odoo.conf`; round-trip + GC validated |
| M4 | S3 backend; env-var secret resolution; force-DB hot-asset rules |
| M5 | NixOS module with templated config + secrets (sops/age) |
| M6 | Multi-tenant `{db_name}` isolation validated |
| M7 | Migration runbook + `force_storage` backfill rehearsed on a prod-like dump |

## 13. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| `gcsfs`/`s3fs` version skew vs. `fsspec` in the chosen nixpkgs pin | Pin nixpkgs; assert compatible versions in M1; the three move together |
| GC mis-scoped against a shared/prod bucket deletes live objects | Dedicated bucket per environment; rehearse GC in staging; back up before first prod GC |
| Latency regression on thumbnail-heavy views | force-DB rules (§8.3) + x-sendfile; load-test kanban views |
| Pre-`fs_attachment` asset residue on local disk | Expected and documented; keep a small persistent volume; do not delete |
| Credentials leak into Nix store or DB | Env-var `$` resolution + systemd credentials/sops; never inline secrets in config tracked in git |
| OCA `18.0` branch churn | Pin to a reviewed commit; vendor via fetchFromGitHub, not a floating ref |

## 14. Open Questions

1. Target Odoo version exactly — 18.0 confirmed against the live OCA `18.0` branch, or pin to 17.0 if a client environment lags?
2. GKE/Workload Identity available for GCS, or do we need SA-JSON delivery for on-prem/GCE?
3. Do any client modules write attachments through non-standard paths that bypass `ir.attachment` (rare, but worth an audit before cutover)?
4. Single shared bucket with `{db_name}` prefixes vs. bucket-per-tenant — driven by client IAM/billing isolation requirements.

## 15. Appendix: Decision Record

**Chosen:** OCA `fs_storage` + `fs_attachment` (fsspec-based), Nix-packaged, config-file-driven.
**Why over Camptocamp `attachment_s3`:** native GCS (not S3-compat), current Odoo branch, single module family for all protocols, declarative-config-first.
**Why over paid Apps-Store modules:** AGPL license clarity, controllable dependency closure, no per-seat cost, fits a solo Nix-packaged stack.
**Why not DB storage:** does not scale; retained only as a force-DB optimization for tiny hot assets.
