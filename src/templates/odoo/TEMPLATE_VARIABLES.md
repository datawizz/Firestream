# Odoo Addons Template Variables

## Required Fields

| Variable | Type | Description |
|---|---|---|
| `project_name` | String | Display name for the project (e.g., "Acme Odoo Addons") |

## Optional Fields

| Variable | Type | Default | Description |
|---|---|---|---|
| `description` | String | `"Custom Odoo addons"` | Project description |
| `organization` | String | `""` | Organization name |
| `author` | String | `""` | Addon author name |
| `author_email` | String | `""` | Author email address |
| `license` | String | `"LGPL-3"` | Addon license identifier |
| `odoo_version` | u8 | `18` | Odoo major version: 15, 16, 17, or 18 |
| `odoo_http_port` | u16 | `8069` | Host port mapped to Odoo HTTP |
| `odoo_longpolling_port` | u16 | `8072` | Host port mapped to Odoo longpolling |
| `odoo_admin_email` | String | `"admin"` | Odoo admin login email |
| `odoo_admin_password` | String | `"admin"` | Odoo admin password |
| `postgresql_version` | String | `"17"` | PostgreSQL version (16 or 17) |
| `postgresql_port` | u16 | `5432` | Host port mapped to PostgreSQL |
| `database_name` | String | `"firestream_odoo"` | PostgreSQL database name |
| `database_user` | String | `"firestream"` | PostgreSQL user |
| `database_password` | String | `"firestream"` | PostgreSQL password |
| `firestream_ref` | String | `"github:Cogent-Creation-Co/Firestream/nightly"` | Nix flake input URL for Firestream |
| `demo_data` | bool | `false` | Whether to load Odoo demo data on init |

## Derived Fields (Computed Automatically)

| Variable | Source | Example |
|---|---|---|
| `project_slug` | kebab-case of `project_name` | `"acme-odoo-addons"` |
| `odoo_version_full` | `"{odoo_version}.0"` | `"18.0"` |
| `python_version` | Mapped from `odoo_version` | `"3.12"` |

### Python Version Mapping

| Odoo Version | Python Version | Nix Package |
|---|---|---|
| 15 | 3.10 | `pkgs.python310` |
| 16 | 3.10 | `pkgs.python310` |
| 17 | 3.11 | `pkgs.python311` |
| 18 | 3.12 | `pkgs.python312` |
