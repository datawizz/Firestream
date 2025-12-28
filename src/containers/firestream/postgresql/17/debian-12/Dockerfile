# Firestream PostgreSQL 17 - Nix-based Docker Image
# Multi-stage build: Nix builder -> minimal runtime
# Uses Firestream module system for container generation

FROM nixos/nix:latest AS builder

RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build

# Copy the container source (module.nix, scripts/, flake.nix)
COPY src/containers/firestream/postgresql/flake.nix ./
COPY src/containers/firestream/postgresql/module.nix ./
COPY src/containers/firestream/postgresql/scripts/ ./scripts/

# Copy the Firestream module system
COPY bin/nix/firestream/ ./bin/nix/firestream/

# Copy top-level flake for workspace imports
COPY flake.nix ./flake.nix
COPY flake.lock ./flake.lock

# Build the PostgreSQL 17 runtime environment
RUN nix build .#runtimeEnv17 --no-link

# Export the closure to a local store
RUN mkdir -p /nix-export && \
    nix copy --no-check-sigs --to /nix-export .#runtimeEnv17

# Runtime stage - minimal base
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="postgresql" \
      org.opencontainers.image.version="17" \
      org.opencontainers.image.description="Firestream PostgreSQL 17 built with Nix module system"

# Copy Nix store from builder
COPY --from=builder /nix-export/nix /nix

# Create symlink to runtime environment
RUN ln -s /nix/store/*-postgresql-runtime-env /opt/nix-env

# Create postgres user (UID 1001 to match Bitnami)
RUN groupadd -g 1001 postgres && \
    useradd -u 1001 -g 1001 -m -d /opt/bitnami/postgresql postgres

# Create required directories with proper permissions
RUN mkdir -p /opt/bitnami/postgresql/conf \
             /opt/bitnami/postgresql/conf/conf.d \
             /opt/bitnami/postgresql/conf.default \
             /opt/bitnami/postgresql/logs \
             /opt/bitnami/postgresql/tmp \
             /bitnami/postgresql \
             /bitnami/postgresql/data \
             /docker-entrypoint-initdb.d \
             /docker-entrypoint-preinitdb.d \
             /run/postgresql \
             /firestream/postgresql/.state && \
    chmod -R 775 /opt/bitnami/postgresql /bitnami /run/postgresql /firestream && \
    chown -R 1001:1001 /opt/bitnami/postgresql /bitnami /docker-entrypoint-initdb.d /docker-entrypoint-preinitdb.d /run/postgresql /firestream

# Environment variables
ENV POSTGRESQL_BASE_DIR="/opt/bitnami/postgresql" \
    POSTGRESQL_VOLUME_DIR="/bitnami/postgresql" \
    PATH="/opt/nix-env/bin:$PATH" \
    LANG="en_US.UTF-8" \
    LC_ALL="C.UTF-8"

WORKDIR /opt/bitnami/postgresql
USER 1001

EXPOSE 5432

# Volumes for persistence
VOLUME ["/bitnami/postgresql", "/docker-entrypoint-initdb.d", "/docker-entrypoint-preinitdb.d"]

ENTRYPOINT ["/opt/nix-env/bin/postgresql-entrypoint"]
CMD []
