# Template Structure

This directory contains Tera templates used for generating Superset YAML configuration files.

## Template Files

- `metadata.yaml.tera` - Basic metadata for the dashboard export
- `database.yaml.tera` - Database connection configuration
- `dataset.yaml.tera` - Dataset definitions including columns and metrics
- `chart.yaml.tera` - Chart/visualization configurations
- `dashboard.yaml.tera` - Dashboard layout and settings

## Template Format

All templates follow the naming convention: `{filename}.{extension}.tera`

This allows the generated files to have the correct extension (`.yaml`) while being clearly identified as Tera templates.

## Usage

Templates are loaded at compile time using `include_str!()` macro in `src/templates.rs`. This ensures templates are embedded in the binary and don't need to be distributed separately.

## Modifying Templates

When modifying templates:
1. Edit the `.tera` file in this directory
2. Templates are automatically reloaded on next compilation
3. Use Tera syntax for templating (https://tera.netlify.app/)

## Variables

Each template expects specific variables to be provided in the Tera context. See the template files themselves for the required variables and their usage.
