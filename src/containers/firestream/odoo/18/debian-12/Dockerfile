# Firestream Odoo - Nix-based Docker Image
# Multi-stage build: Nix builder -> minimal runtime

FROM nixos/nix:latest AS builder

RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build
COPY src/containers/firestream/odoo/flake.nix src/containers/firestream/odoo/flake.lock src/containers/firestream/odoo/pyproject.toml src/containers/firestream/odoo/uv.lock ./

# Build the unified runtime environment (includes everything: Python, wkhtmltopdf, nodejs, etc.)
RUN nix build .#runtimeEnv --no-link

# Export the closure to a local store (--no-check-sigs for locally-built paths)
RUN mkdir -p /nix-export && \
    nix copy --no-check-sigs --to /nix-export .#runtimeEnv

# Runtime stage - minimal base
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="odoo" \
      org.opencontainers.image.version="18.0" \
      org.opencontainers.image.description="Firestream Odoo built with Nix" \
      org.opencontainers.image.vendor="Firestream" \
      org.opencontainers.image.source="https://github.com/Cogent-Creation-Co/Firestream"

# Copy Nix store from builder
COPY --from=builder /nix-export/nix /nix

# Create odoo user (UID 1001 to match Bitnami)
RUN useradd -u 1001 -g 0 -m -d /home/odoo odoo

# Create writable directory structure
# /opt/nix-env will be symlink to unified runtime (read-only)
# /var/odoo contains writable data directories
RUN mkdir -p /var/odoo/custom-addons \
             /var/odoo/data \
             /var/odoo/logs \
             /var/odoo/tmp \
             /var/odoo/config \
             /bitnami/odoo \
             /bitnami/python \
             /docker-entrypoint-init.d

# Single symlink to unified runtime environment (no more glob loops!)
# The runtimeEnv buildEnv contains all binaries in /bin and Odoo source in /opt
RUN ln -s /nix/store/*-odoo-runtime-env /opt/nix-env && \
    ln -s /opt/nix-env/opt/odoo /opt/odoo

# Set permissions on writable directories
RUN chmod -R 775 /var/odoo /bitnami && \
    chown -R odoo:root /var/odoo /bitnami

# Environment variables
# PATH points to unified runtime - includes python, wkhtmltopdf, wkhtmltoimage, nodejs, etc.
ENV PATH="/opt/nix-env/bin:$PATH" \
    ODOO_HOME="/opt/odoo" \
    ODOO_DATA_DIR="/var/odoo/data" \
    ODOO_ADDONS_DIR="/var/odoo/custom-addons" \
    ODOO_CONFIG_DIR="/var/odoo/config" \
    ODOO_LOG_DIR="/var/odoo/logs" \
    PYTHONPYCACHEPREFIX="/var/odoo/tmp/pycache" \
    LANG="C.UTF-8" \
    LC_ALL="C.UTF-8"

WORKDIR /opt/odoo
USER 1001

EXPOSE 8069 8072

# Volumes for persistence (writable directories under /var/odoo)
VOLUME ["/var/odoo/custom-addons", "/var/odoo/data", "/bitnami/odoo", "/bitnami/python"]

ENTRYPOINT ["/opt/nix-env/bin/entrypoint"]
CMD ["odoo"]
