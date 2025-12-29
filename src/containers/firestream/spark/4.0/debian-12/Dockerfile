# Copyright Firestream. Apache-2.0 License.
# SPDX-License-Identifier: APACHE-2.0
#
# Apache Spark Container - Two-Stage Nix Build
#
# This Dockerfile uses Nix to build a reproducible Spark container.
# Stage 1: Build the Nix closure containing all dependencies
# Stage 2: Copy the closure into a minimal Debian runtime
#
# Build: docker build -t firestream-spark:4.0.0 .
# Run:   docker run -e SPARK_MODE=master firestream-spark:4.0.0

# ============================================================
# Stage 1: Nix Builder
# ============================================================
FROM nixos/nix:latest AS builder

# Enable flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build

# Copy Nix files first (for better layer caching)
COPY flake.nix flake.lock* ./
COPY module.nix ./
COPY scripts/ scripts/

# Build the runtime environment
# This creates a Nix closure with all dependencies
RUN nix build .#runtimeEnv --out-link /nix-export 2>&1 || \
    (echo "Build failed, falling back to Docker image package" && \
     nix build .#dockerImage --out-link /nix-export)

# Export the Nix store paths needed at runtime
RUN mkdir -p /nix-closure && \
    nix-store --export $(nix-store -qR /nix-export) > /nix-closure/closure.nar

# ============================================================
# Stage 2: Minimal Runtime
# ============================================================
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="firestream-spark" \
      org.opencontainers.image.description="Apache Spark built with Nix for reproducibility" \
      org.opencontainers.image.version="4.0.0" \
      org.opencontainers.image.vendor="Firestream" \
      org.opencontainers.image.source="https://github.com/Cogent-Creation-Co/Firestream"

# Install Nix store import dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    locales \
    && rm -rf /var/lib/apt/lists/*

# Set up locale
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && \
    locale-gen
ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

# Copy the Nix store from builder
COPY --from=builder /nix /nix
COPY --from=builder /nix-export /opt/nix-env

# Create spark user (UID 1001 for Bitnami compatibility)
RUN groupadd -r -g 1001 spark && \
    useradd -r -u 1001 -g spark -d /home/spark -s /bin/bash -c "Spark User" spark && \
    mkdir -p /home/spark && \
    chown spark:spark /home/spark

# Create directory structure
RUN mkdir -p \
    /opt/spark \
    /opt/spark/conf \
    /opt/spark/conf.default \
    /opt/spark/work \
    /opt/spark/logs \
    /opt/spark/tmp \
    /opt/spark/jars \
    /firestream/spark/data \
    /firestream/spark/jars \
    /firestream/spark/.state \
    /docker-entrypoint-initdb.d \
    /bitnami \
    && chown -R spark:spark /opt/spark /firestream/spark /home/spark \
    && chmod 1777 /opt/spark/tmp

# Link Nix environment
RUN ln -sf /opt/nix-env /opt/bitnami 2>/dev/null || true

# Environment variables
ENV PATH="/opt/nix-env/bin:/opt/spark/bin:/opt/spark/sbin:${PATH}" \
    SPARK_HOME="/opt/spark" \
    SPARK_CONF_DIR="/opt/spark/conf" \
    JAVA_HOME="/opt/nix-env" \
    HOME="/home/spark" \
    USER="spark" \
    # Bitnami compatibility
    BITNAMI_ROOT_DIR="/opt/bitnami" \
    BITNAMI_VOLUME_DIR="/bitnami" \
    BITNAMI_APP_NAME="spark" \
    APP_VERSION="4.0.0" \
    # Firestream paths
    FIRESTREAM_ROOT_DIR="/opt/firestream" \
    FIRESTREAM_VOLUME_DIR="/firestream" \
    # Spark defaults
    SPARK_MODE="master" \
    SPARK_USER="spark" \
    SPARK_DAEMON_USER="spark" \
    SPARK_DAEMON_GROUP="spark" \
    # Python/PySpark
    PYTHONPATH="/opt/spark/python:/opt/spark/python/lib/py4j-0.10.9.7-src.zip" \
    PYSPARK_PYTHON="python3" \
    PYSPARK_DRIVER_PYTHON="python3" \
    # SSL defaults
    SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt" \
    NIX_SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"

# Expose ports
# 7077: Spark master
# 8080: Spark master UI
# 8081: Spark worker UI
# 4040: Spark application UI
# 6066: REST submission port
EXPOSE 7077 8080 8081 4040 6066

# Data volumes
VOLUME ["/opt/spark/work", "/opt/spark/logs", "/firestream/spark/data", "/firestream/spark/jars"]

# Run as non-root
USER 1001

WORKDIR /opt/spark

# Entrypoint from Nix build
ENTRYPOINT ["/opt/nix-env/bin/spark-entrypoint"]

# Default command: run Spark master
CMD ["spark-class", "org.apache.spark.deploy.master.Master"]
