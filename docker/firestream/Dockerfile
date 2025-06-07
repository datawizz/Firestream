# Build Arguments #

ARG DEPLOYMENT_MODE
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS
ARG HOST_KVM_STATUS
ARG HOST_USERNAME
ARG DOCKER_BUILDKIT=1

## Dependency Container ##

# Combines downloading of external resources in one place.
# Allows for efficient multistage build with minimum network activity
# and assertation of remote resources by good 'ol hard coded hash matching
# or version pinning where possible.

FROM debian:bookworm as dependencies

ENV DEBIAN_FRONTEND=noninteractive
ARG DOCKER_BUILDKIT

ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS

# Install the packages but delete the package lists to enable caching of the layer
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    wget \
    unzip \
    zip \
    curl \
    gnupg2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*


WORKDIR /tmp/build/workspace


### Devcontainer ###

# Contains everything (and the kitchen sink) required to build project artifacts and run tests

FROM debian:bookworm as devcontainer

ENV DEBIAN_FRONTEND=noninteractive

## Build Args ##
ARG DEPLOYMENT_MODE
ARG HOST_USERNAME
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_MACHINE_ID
ARG HOST_IP
ARG HOST_DOCKER_GID
ARG HOST_GPU_STATUS

# Install additional OS packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # required for Python and Node interfaces to Kafka
    librdkafka-dev \
    librdkafka++1 \
    librdkafka1 \
    build-essential \
    iputils-ping \
    dnsutils \
    apt-transport-https \
    ca-certificates \
    gnupg \
    stress \
    # netcat \
    postgresql-client \
    libglib2.0-0 \
    libnss3 \
    libx11-6 \
    # Headless chrome
    chromium \
    # Dev Tools
    wget \
    unzip \
    zip \
    curl \
    gnupg2 \
    ca-certificates \
    # Used to enable GPU support in the container
    pciutils \
    # pyhive Thrift API dependencies
    libsasl2-dev \
    libsasl2-modules-gssapi-mit \
    # Required for TLS Websocket support in Rust compiler
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    #Airflow dependencies
    apt-transport-https \
    apt-utils ca-certificates \
    curl \
    dumb-init \
    freetds-bin \
    gosu \
    krb5-user \
    libgeos-dev \
    ldap-utils \
    libsasl2-2 \
    libsasl2-modules \
    libxmlsec1 \
    locales \
    libffi8 \
    libldap-2.5-0 \
    libssl3 \
    netcat-openbsd \
    lsb-release \
    openssh-client \
    rsync \
    sasl2-bin \
    sqlite3 \
    sudo \
    unixodbc \
    && rm -rf /var/lib/apt/lists/*


# Add the user and docker group
RUN groupadd -g ${HOST_GROUP_ID} ${HOST_USERNAME} || true \
    && groupadd -g ${HOST_DOCKER_GID:-997} docker || true \
    && useradd -m -d /home/${HOST_USERNAME} -u ${HOST_USER_ID} -g ${HOST_GROUP_ID} -s /bin/bash -o ${HOST_USERNAME} || true \
    && usermod -aG docker ${HOST_USERNAME} || true \
    && chown -R ${HOST_USER_ID}:${HOST_GROUP_ID} /home/${HOST_USERNAME}


# [Option] Install zsh
ARG INSTALL_ZSH="true"
# [Option] Upgrade OS packages to their latest versions
ARG UPGRADE_PACKAGES="false"

# Install VS Code remote development container features
COPY bin/install_scripts/common-debian.sh /tmp/workspace/bin/install_scripts/common-debian.sh
RUN /bin/bash /tmp/workspace/bin/install_scripts/common-debian.sh "${INSTALL_ZSH}" "${HOST_USERNAME}" "${HOST_USER_ID}" "${HOST_GROUP_ID}" "${UPGRADE_PACKAGES}" "true" "true"



USER root



############
# NIX ####
############

# Install Nix as the non-root user
ENV NIXPKGS_ALLOW_UNFREE=1
ENV NIX_VERSION="nixos.org.2.24.9"
WORKDIR /home/$HOST_USERNAME

# Prep all directories as root
# Install Nix as the non-root user

COPY bin/install_scripts/nix_setup.sh nix_setup.sh
COPY bin/install_scripts/nix_install.sh nix_install.sh

# Set environment variables to include Nix binaries in PATH
ENV PATH=/home/$HOST_USERNAME/.nix-profile/bin:/home/vscode/.nix-profile/sbin:$PATH

# Create the Nix configuration directory
RUN mkdir -p /home/$HOST_USERNAME/.config/nix
RUN chown -R $HOST_USER_ID:$HOST_GROUP_ID /home/$HOST_USERNAME/
# Configure Nix to change the download buffer size to 500M
RUN grep -q 'download-buffer-size' /home/$HOST_USERNAME/.config/nix/nix.conf || echo 'download-buffer-size = 500M' >> /home/$HOST_USERNAME/.config/nix/nix.conf
RUN echo 'experimental-features = nix-command flakes' >> /home/$HOST_USERNAME/.config/nix/nix.conf

RUN mkdir -p /nix && chown ${HOST_USERNAME}:${HOST_GROUP_ID} /nix

USER $HOST_USERNAME
RUN /bin/bash /home/$HOST_USERNAME/nix_install.sh

# Copy the flake files
COPY flake.nix flake.nix
COPY flake.lock flake.lock
COPY rust-toolchain.toml rust-toolchain.toml
COPY bin/nix bin/nix

# Build the flake
RUN nix build .#container

USER root
# Run the setup script so paths are correctly set at login
RUN /bin/bash /home/$HOST_USERNAME/result/bin/setup-container


# ### Python ###
# COPY ./requirements.txt /tmp/build/workspace/requirements.txt
# RUN python -m pip install --user --break-system-packages -r /tmp/workspace/build/requirements.txt

# Install PySpark and its dependencies (py4j) which are bundled with Spark
# RUN cd /opt/spark/python \
#     && python -m pip install --no-warn-script-location --no-deps --no-index --find-links file:///tmp/python_packages py4j \
#     && python -m pip install .




### Cleanup ###
USER root
RUN rm -rf /tmp/workspace
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Required for Docker-from-Docker
# Create a wrapper script that handles the case where docker-init.sh doesn't exist
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'if [ -f /usr/local/share/docker-init.sh ]; then' >> /entrypoint.sh && \
    echo '    exec /usr/local/share/docker-init.sh "$@"' >> /entrypoint.sh && \
    echo 'else' >> /entrypoint.sh && \
    echo '    echo "Warning: docker-init.sh not found, running command directly"' >> /entrypoint.sh && \
    echo '    exec "$@"' >> /entrypoint.sh && \
    echo 'fi' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

USER ${HOST_USERNAME}
WORKDIR /workspace

# Set ENTRYPOINT to the wrapper script
ENTRYPOINT ["/entrypoint.sh"]
CMD ["sleep", "infinity"]
