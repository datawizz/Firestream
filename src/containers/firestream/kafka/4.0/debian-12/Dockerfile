# Firestream Kafka 4.0 - Nix-based Docker Image
# Multi-stage build: Nix builder -> minimal runtime
# Uses Firestream module system for container generation
#
# Note: Kafka 4.0+ uses KRaft mode only (no ZooKeeper dependency)

FROM nixos/nix:latest AS builder

RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build

# Copy the container source (module.nix, scripts/, flake.nix)
COPY src/containers/firestream/kafka/flake.nix ./
COPY src/containers/firestream/kafka/module.nix ./
COPY src/containers/firestream/kafka/scripts/ ./scripts/

# Copy the Firestream module system
COPY bin/nix/firestream/ ./bin/nix/firestream/

# Copy top-level flake for workspace imports
COPY flake.nix ./flake.nix
COPY flake.lock ./flake.lock

# Build the Kafka 4.0 runtime environment
RUN nix build .#runtimeEnv --no-link

# Export the closure to a local store
RUN mkdir -p /nix-export && \
    nix copy --no-check-sigs --to /nix-export .#runtimeEnv

# Runtime stage - minimal base
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="kafka" \
      org.opencontainers.image.version="4.0" \
      org.opencontainers.image.description="Firestream Kafka 4.0 built with Nix module system (KRaft mode)"

# Copy Nix store from builder
COPY --from=builder /nix-export/nix /nix

# Create symlink to runtime environment
RUN ln -s /nix/store/*-kafka-runtime-env /opt/nix-env

# Create kafka user (UID 1001 to match Bitnami)
RUN groupadd -g 1001 kafka && \
    useradd -u 1001 -g 1001 -m -d /opt/bitnami/kafka kafka

# Create required directories with proper permissions
RUN mkdir -p /opt/bitnami/kafka/config \
             /opt/bitnami/kafka/config/certs \
             /opt/bitnami/kafka/config.default \
             /opt/bitnami/kafka/logs \
             /opt/bitnami/kafka/tmp \
             /bitnami/kafka \
             /bitnami/kafka/data \
             /bitnami/kafka/config \
             /docker-entrypoint-initdb.d \
             /firestream/kafka/.state && \
    chmod -R 775 /opt/bitnami/kafka /bitnami /firestream && \
    chown -R 1001:1001 /opt/bitnami/kafka /bitnami /docker-entrypoint-initdb.d /firestream

# Environment variables
ENV KAFKA_BASE_DIR="/opt/bitnami/kafka" \
    KAFKA_VOLUME_DIR="/bitnami/kafka" \
    KAFKA_HOME="/opt/bitnami/kafka" \
    PATH="/opt/nix-env/bin:$PATH" \
    BITNAMI_APP_NAME="kafka" \
    BITNAMI_IMAGE_VERSION="4.0" \
    LANG="en_US.UTF-8" \
    LC_ALL="C.UTF-8"

WORKDIR /opt/bitnami/kafka
USER 1001

EXPOSE 9092 9093

# Volumes for persistence
VOLUME ["/bitnami/kafka", "/docker-entrypoint-initdb.d"]

ENTRYPOINT ["/opt/nix-env/bin/kafka-entrypoint"]
CMD []
