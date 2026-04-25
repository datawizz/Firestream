# Services

Backend services and microservices for the application.

## Purpose

This directory contains standalone backend services that can be deployed independently. Services communicate via well-defined APIs (REST, gRPC, or WebSocket).

## Structure

```
src/service/
├── README.md           # This file
├── api-gateway/        # API gateway service (example)
│   ├── Cargo.toml      # Rust service
│   └── src/
├── auth-service/       # Authentication service (example)
│   └── package.json    # Node.js service
└── ml-inference/       # ML inference service (example)
    └── pyproject.toml  # Python service
```

## Creating a New Service

### Rust Service
```bash
mkdir -p src/service/my-service
cd src/service/my-service
cargo init --name my-service
```

Then add to the root `Cargo.toml` workspace members.

### Node.js Service
```bash
mkdir -p src/service/my-service
cd src/service/my-service
pnpm init
```

Then add to `pnpm-workspace.yaml`.

### Python Service
```bash
mkdir -p src/service/my-service
cd src/service/my-service
# Create pyproject.toml with UV
```

Then add to root `pyproject.toml` workspace members.

## Best Practices

1. **Single Responsibility**: Each service should have one clear purpose
2. **API First**: Define your API contract before implementation
3. **Health Checks**: Implement `/health` endpoint for orchestration
4. **Configuration**: Use environment variables for configuration
5. **Logging**: Use structured logging (JSON format)
6. **Testing**: Include unit and integration tests
