# Firestream Airflow - Nix-based Docker Image
# The Nix flake is the source of truth for all logic.
# This Dockerfile is a minimal wrapper.

FROM nixos/nix:latest AS builder

RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build
COPY src/containers/firestream/airflow/flake.nix \
     src/containers/firestream/airflow/flake.lock \
     src/containers/firestream/airflow/pyproject.toml \
     src/containers/firestream/airflow/uv.lock ./

# Build the complete runtime environment
RUN nix build .#runtimeEnv --no-link && \
    mkdir -p /nix-export && \
    nix copy --no-check-sigs --to /nix-export .#runtimeEnv

# Runtime stage - minimal base
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="airflow" \
      org.opencontainers.image.version="3.0.3" \
      org.opencontainers.image.description="Firestream Airflow - Nix-first build" \
      org.opencontainers.image.vendor="Firestream" \
      org.opencontainers.image.source="https://github.com/Cogent-Creation-Co/Firestream"

# Copy Nix store from builder
COPY --from=builder /nix-export/nix /nix

# Create airflow user (UID 1001 for Bitnami compatibility)
RUN useradd -u 1001 -g 0 -m -d /opt/airflow airflow && \
    mkdir -p /bitnami/python && \
    chown -R airflow:root /bitnami

# Symlink to runtime environment
RUN ln -s /nix/store/*-airflow-runtime-env /opt/nix-env

# Minimal environment - directories created at runtime by entrypoint
ENV PATH="/opt/nix-env/bin:$PATH" \
    AIRFLOW_HOME="/opt/airflow" \
    LANG="C.UTF-8" \
    LC_ALL="C.UTF-8"

WORKDIR /opt/airflow
USER 1001

EXPOSE 8080 8125 8793 8794

# Volumes for persistence
VOLUME ["/opt/airflow/dags", "/opt/airflow/logs", "/opt/airflow/plugins", "/bitnami/python"]

ENTRYPOINT ["/opt/nix-env/bin/entrypoint"]
