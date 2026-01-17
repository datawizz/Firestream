# K8s-Manager Integration Tests

This directory contains integration tests for the k8s-manager tool, focusing on Docker image build and push functionality with k3d registries.

## Running Integration Tests

Integration tests require Docker and k3d to be installed and running. They are disabled by default to avoid interfering with regular unit tests.

### Run all integration tests:
```bash
cargo test --features integration_tests
```

### Run specific test file:
```bash
cargo test --features integration_tests docker_integration_tests
```

### Run with output for debugging:
```bash
cargo test --features integration_tests -- --nocapture
```

### Run a specific test:
```bash
cargo test --features integration_tests test_build_and_push_to_k3d -- --nocapture
```

## Test Files

- `k3d_tests.rs` - Tests for k3d cluster management functionality
- `docker_integration_tests.rs` - Tests for Docker image build/push functionality
- `image_tests.rs` - Basic CLI tests for image commands

## Docker Integration Tests

The `docker_integration_tests.rs` file contains comprehensive tests for:

1. **Image Building**
   - Basic image builds
   - Custom Dockerfile support
   - Build arguments
   - No-cache builds

2. **Registry Push**
   - Push to k3d registry
   - Auto-detection of k3d registry
   - Multiple registry scenarios
   - Error handling

3. **Build-and-Push Workflow**
   - Combined build and push operations
   - Integration with k3d cluster deployment

## Test Requirements

- Docker daemon must be running
- k3d must be installed
- User must have permissions to run Docker commands
- Sufficient disk space for test images and clusters

## Clean Up

Tests are designed to clean up after themselves, but if tests fail, you may need to manually clean up:

```bash
# Remove test clusters
k3d cluster list | grep test- | awk '{print $1}' | xargs -r k3d cluster delete

# Remove test images
docker images | grep test- | awk '{print $3}' | xargs -r docker rmi -f

# Remove test registries
k3d registry list | grep test- | awk '{print $1}' | xargs -r k3d registry delete
```

## Troubleshooting

1. **Permission Denied**: Ensure your user is in the docker group
2. **Port Already in Use**: Tests use specific ports (15000-15003, 16550-16552). Ensure these are free
3. **Cluster Creation Timeout**: Increase the sleep duration in tests if clusters take longer to start
4. **Registry Push Failures**: Check Docker daemon logs and ensure registries are accessible