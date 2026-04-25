# Python Libraries

Python packages for data processing, ML, utilities, and scripting.

## Purpose

This directory contains Python packages that provide functionality for:
- Data processing and ETL
- Machine learning and AI
- Scripting and automation
- Integration with external services

## Structure

```
src/lib/python/
├── README.md           # This file
├── data-utils/         # Data processing utilities
│   ├── pyproject.toml
│   └── src/
│       └── data_utils/
│           └── __init__.py
├── ml-models/          # Machine learning models
│   ├── pyproject.toml
│   └── src/
│       └── ml_models/
│           └── __init__.py
└── integrations/       # External service integrations
    └── ...
```

## Setup

This project uses [UV](https://github.com/astral-sh/uv) for Python package management.

### Prerequisites

UV is installed via the Nix flake. When you enter the project directory with direnv, UV is available.

### Creating a New Package

1. Create the package directory:
   ```bash
   mkdir -p src/lib/python/my-package/src/my_package
   touch src/lib/python/my-package/src/my_package/__init__.py
   ```

2. Create `pyproject.toml`:
   ```toml
   [project]
   name = "my-package"
   version = "0.1.0"
   requires-python = ">=3.11"
   dependencies = []

   [build-system]
   requires = ["hatchling"]
   build-backend = "hatchling.build"

   [tool.hatch.build.targets.wheel]
   packages = ["src/my_package"]
   ```

3. Add to root `pyproject.toml` workspace:
   ```toml
   [tool.uv.workspace]
   members = [
       "src/lib/python/my-package",
   ]
   ```

4. Sync dependencies:
   ```bash
   uv sync
   ```

## Development

### Running Tests
```bash
# All tests
uv run pytest

# Specific package
uv run pytest src/lib/python/my-package/
```

### Linting and Formatting
```bash
# Format with black
uv run black src/lib/python/

# Lint with ruff
uv run ruff check src/lib/python/

# Type check with mypy
uv run mypy src/lib/python/
```

### Adding Dependencies
```bash
# Add to specific package
cd src/lib/python/my-package
uv add requests

# Add dev dependency to workspace
uv add --dev pytest
```

## Integration with Rust

Python packages can call Rust code via PyO3:

```python
# In Python
from rust_bindings import my_rust_function

result = my_rust_function(data)
```

Build Rust Python bindings with maturin:
```bash
cd src/lib/rust/python-bindings
maturin develop
```

## Best Practices

1. **Type Hints**: Use type hints for all public functions
2. **Docstrings**: Document modules, classes, and functions
3. **Testing**: Maintain >80% test coverage
4. **Dependencies**: Minimize dependencies, prefer stdlib
5. **Async**: Use async/await for I/O-bound operations
