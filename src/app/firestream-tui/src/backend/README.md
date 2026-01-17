# Backend Architecture

The Firestream TUI uses a trait-based backend system that allows switching between different data sources:

## Backend Implementations

### `MockClient`
- Default backend for development
- Returns hardcoded sample data
- No external dependencies
- Used when `LOCAL_DATA_DIRECTORY` is not set

### `IcebergBackend`
- Provides real Apache Iceberg data operations
- Uses the `IcebergService` to interact with Iceberg tables
- Automatically selected when `LOCAL_DATA_DIRECTORY` environment variable is set
- Falls back to mock data for non-Iceberg resources (deployments, templates, etc.)

### `ApiClient`
- Placeholder for future HTTP-based backend
- Will connect to the Firestream API server
- Currently not implemented

## Backend Selection

The backend is automatically selected in `App::new()`:
- If `LOCAL_DATA_DIRECTORY` is set → `IcebergBackend`
- Otherwise → `MockClient`

This allows seamless switching between mock and real data without code changes.
