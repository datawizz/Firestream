# Firestream JupyterHub Container
# Multi-stage build: Nix -> Debian minimal runtime
#
# Copyright Firestream. Apache-2.0 License.

# =============================================================================
# Stage 1: Nix builder
# =============================================================================
FROM nixos/nix:latest AS builder

# Enable flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Copy only files needed for the build
WORKDIR /build
COPY src/containers/firestream/jupyterhub/flake.nix src/containers/firestream/jupyterhub/flake.lock src/containers/firestream/jupyterhub/pyproject.toml src/containers/firestream/jupyterhub/uv.lock ./
COPY src/containers/firestream/jupyterhub/scripts/ ./scripts/
COPY src/containers/firestream/jupyterhub/module.nix ./

# Copy the Firestream module system (relative path for standalone builds)
COPY bin/nix/firestream /build-context/bin/nix/firestream

# Build the unified runtime environment
RUN nix build .#runtimeEnv --no-link --print-build-logs

# Export to a directory for the runtime stage (--no-check-sigs for locally-built paths)
RUN mkdir -p /nix-export && \
    nix copy --no-check-sigs --to /nix-export .#runtimeEnv

# =============================================================================
# Stage 2: Runtime (minimal Debian)
# =============================================================================
FROM debian:bookworm-slim

LABEL maintainer="Firestream <support@firestream.io>" \
      org.opencontainers.image.title="Firestream JupyterHub" \
      org.opencontainers.image.version="5.3.0" \
      org.opencontainers.image.description="Multi-user Jupyter notebook server" \
      org.opencontainers.image.vendor="Firestream" \
      org.opencontainers.image.source="https://github.com/Cogent-Creation-Co/Firestream"

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    locales \
    netcat-openbsd \
    openssl \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
    && locale-gen

ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

# Copy Nix store from builder
COPY --from=builder /nix-export/nix /nix

# Create jupyterhub user (Bitnami compatible UID 1001)
RUN groupadd -r -g 1001 jupyterhub && \
    useradd -r -u 1001 -g jupyterhub -d /opt/bitnami/jupyterhub -s /bin/bash jupyterhub

# Create directory structure (matching Bitnami conventions)
RUN mkdir -p \
    /opt/bitnami/jupyterhub/etc \
    /opt/bitnami/jupyterhub/data \
    /opt/bitnami/jupyterhub/log \
    /opt/bitnami/jupyterhub/tmp \
    /opt/bitnami/scripts/jupyterhub \
    /opt/bitnami/scripts/jupyterhub/bitnami-templates \
    /bitnami/jupyterhub \
    /docker-entrypoint-init.d \
    /firestream/jupyterhub/.state

# Copy rootfs (scripts, templates, etc.)
COPY src/containers/firestream/jupyterhub/5/debian-12/rootfs/ /

# Set up symlink to Nix runtime (single glob to find the runtime env)
RUN ln -s /nix/store/*-jupyterhub-runtime-env /opt/nix-env && \
    chmod +x /opt/bitnami/scripts/jupyterhub/*.sh \
             /opt/bitnami/scripts/*.sh \
             /post-init.sh 2>/dev/null || true

# Set ownership
RUN chown -R 1001:1001 \
    /opt/bitnami/jupyterhub \
    /bitnami/jupyterhub \
    /docker-entrypoint-init.d \
    /firestream && \
    chmod -R 775 /opt/bitnami/jupyterhub /bitnami/jupyterhub

# Environment
ENV PATH="/opt/nix-env/bin:$PATH" \
    BITNAMI_APP_NAME="jupyterhub" \
    APP_VERSION="5.3.0" \
    JUPYTERHUB_BASE_DIR="/opt/bitnami/jupyterhub" \
    JUPYTERHUB_CONF_DIR="/opt/bitnami/jupyterhub/etc" \
    JUPYTERHUB_DATA_DIR="/opt/bitnami/jupyterhub/data" \
    JUPYTERHUB_LOGS_DIR="/opt/bitnami/jupyterhub/log" \
    JUPYTERHUB_TMP_DIR="/opt/bitnami/jupyterhub/tmp"

WORKDIR /opt/bitnami/jupyterhub
USER 1001

EXPOSE 8000 8081

VOLUME ["/bitnami/jupyterhub", "/docker-entrypoint-init.d"]

ENTRYPOINT ["/opt/bitnami/scripts/jupyterhub/entrypoint.sh"]
CMD ["/opt/bitnami/scripts/jupyterhub/run.sh"]
