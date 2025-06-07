# Firestream Template Module

The template module provides project scaffolding functionality for Firestream, allowing users to quickly create new projects with pre-configured Helm charts, Docker configurations, and application code.

## Features

- **Interactive Project Creation**: Step-by-step prompts to configure your project
- **Non-Interactive Mode**: Use a values file for automated project generation
- **Multiple Project Types**: Support for Python FastAPI, Kubernetes-only, or Docker-only projects
- **Embedded Templates**: All templates are compiled into the binary for easy distribution
- **Helm Chart Generation**: Complete Helm chart with deployment, service, ingress, and more
- **Docker Support**: Dockerfile and build configuration included

## Usage

### Interactive Mode

```bash
firestream template
```

This will prompt you for:
- Project name
- Version
- Description
- Docker image
- Port configuration
- Resource limits
- Ingress settings

### Non-Interactive Mode

Create a values file (e.g., `my-project-values.yaml`):

```yaml
project_name: my-api
version: "1.0.0"
description: "My API Service"
docker_image: "python:3.9-slim"
port: 8080
replicas: 3
cpu_request: "100m"
memory_request: "128Mi"
cpu_limit: "500m"
memory_limit: "512Mi"
enable_ingress: true
ingress_host: "api.example.com"
ingress_path: "/"
```

Then run:

```bash
firestream template --non-interactive --values my-project-values.yaml
```

### Command Options

- `--name`: Set the project name (otherwise prompted)
- `--project-type`: Choose project type (default: `python-fastapi`)
  - `python-fastapi`: Full Python FastAPI project with Helm and Docker
  - `helm`: Helm chart only
  - `kubernetes`: Kubernetes manifests without Docker
  - `docker`: Docker configuration only
- `--output`: Output directory (default: current directory)
- `--non-interactive`: Skip prompts and use defaults or values file
- `--values`: Path to values file for non-interactive mode

## Generated Project Structure

```
my-project/
├── .devcontainer/          # VS Code dev container configuration
│   └── devcontainer.json
├── docker/                 # Docker configuration
│   └── my-project/
│       └── Dockerfile
├── helm/                   # Helm chart
│   └── template/
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── templates/
│       │   ├── deployment.yaml
│       │   ├── service.yaml
│       │   ├── ingress.yaml
│       │   ├── hpa.yaml
│       │   ├── serviceaccount.yaml
│       │   ├── _helpers.tpl
│       │   └── NOTES.txt
│       └── .helmignore
├── src/                    # Application source code
│   └── main.py
├── tests/                  # Test files
│   └── test.py
├── Makefile               # Build and deployment commands
├── README.md              # Project documentation
├── bootstrap.sh           # Bootstrap script
├── entrypoint.sh          # Container entrypoint
└── .dockerignore          # Docker ignore file
```

## Next Steps After Generation

1. **Review Generated Files**: Check the configuration in `helm/template/values.yaml`

2. **Build Docker Image**:
   ```bash
   make build
   ```

3. **Deploy to Kubernetes**:
   ```bash
   make deploy
   ```

4. **Run Locally**:
   ```bash
   python src/main.py
   ```

## Template Customization

Templates are stored in `src/templates/_python_fastapi/` and embedded at compile time. To add new templates:

1. Create a new template directory under `src/templates/`
2. Add the template files
3. Update `embedded.rs` to include the new templates
4. Add support for the new project type in `mod.rs`

## Technical Details

- **Template Engine**: Uses Tera (Jinja2-like) for Chart.yaml and values.yaml
- **Post-Processing**: Helm templates are post-processed to replace placeholder names
- **Binary Embedding**: Templates are embedded using `include_str!` and `include_bytes!`
- **Cross-Platform**: Handles executable permissions on Unix systems

## Examples

### Create a simple API service

```bash
firestream template --name api-service --project-type python-fastapi
```

### Create a batch processing job

```bash
firestream template --name batch-processor --non-interactive \
  --values batch-job-values.yaml
```

### Generate only Helm charts

```bash
firestream template --name my-chart --project-type helm
```
