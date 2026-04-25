# Standard Project Development Container
ARG DEPLOYMENT_MODE=development
ARG HOST_USER_ID=1000
ARG HOST_GROUP_ID=1000
ARG HOST_USERNAME=developer
ARG HOST_DOCKER_GID=999
ARG DOCKER_BUILDKIT=1

# Base image
FROM debian:bookworm as devcontainer

ENV DEBIAN_FRONTEND=noninteractive

# Build arguments
ARG HOST_USERNAME
ARG HOST_USER_ID
ARG HOST_GROUP_ID
ARG HOST_DOCKER_GID

# Install base packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # Basic utilities
    curl \
    wget \
    ca-certificates \
    gnupg2 \
    git \
    openssh-client \
    sudo \
    locales \
    # Build tools
    build-essential \
    pkg-config \
    # Development tools
    vim \
    less \
    htop \
    # Shell
    bash \
    zsh \
    # Required for various tools
    xz-utils \
    libssl-dev \
    zlib1g-dev \
    libbz2-dev \
    libreadline-dev \
    libsqlite3-dev \
    libncurses5-dev \
    libncursesw5-dev \
    libffi-dev \
    liblzma-dev \
    && rm -rf /var/lib/apt/lists/*

# Set locale
RUN echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen && locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

# Create user and groups
RUN groupadd -g ${HOST_GROUP_ID} ${HOST_USERNAME} || true \
    && groupadd -g ${HOST_DOCKER_GID} docker || true \
    && useradd -m -u ${HOST_USER_ID} -g ${HOST_GROUP_ID} -G docker -s /bin/bash ${HOST_USERNAME} || true \
    && echo "${HOST_USERNAME} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Create /nix directory with proper permissions
RUN mkdir -p /nix && chown ${HOST_USERNAME}:${HOST_GROUP_ID} /nix

# Copy Nix installation scripts
COPY --chown=${HOST_USER_ID}:${HOST_GROUP_ID} bin/install_scripts/nix_install.sh /home/${HOST_USERNAME}/nix_install.sh
COPY --chown=${HOST_USER_ID}:${HOST_GROUP_ID} bin/install_scripts/nix_setup.sh /home/${HOST_USERNAME}/nix_setup.sh

# Make scripts executable
RUN chmod +x /home/${HOST_USERNAME}/nix_install.sh /home/${HOST_USERNAME}/nix_setup.sh

# Install Nix as the user
USER ${HOST_USERNAME}
WORKDIR /home/${HOST_USERNAME}

# Set environment variables for Nix installation
ENV NIXPKGS_ALLOW_UNFREE=1
ENV NIX_CONFIG="experimental-features = nix-command flakes"
ENV USERNAME=${HOST_USERNAME}

# Run the Nix setup script
RUN /home/${HOST_USERNAME}/nix_setup.sh

# Set up Nix environment
ENV PATH="/home/${HOST_USERNAME}/.nix-profile/bin:/home/${HOST_USERNAME}/.nix-profile/sbin:$PATH"
ENV NIX_PATH="nixpkgs=/home/${HOST_USERNAME}/.nix-defexpr/channels/nixpkgs"

# Copy project files
USER root
WORKDIR /workspace

# Ensure workspace directory has correct ownership
RUN chown ${HOST_USER_ID}:${HOST_GROUP_ID} /workspace

# Copy flake files first (for better caching)
COPY --chown=${HOST_USER_ID}:${HOST_GROUP_ID} flake.nix flake.lock ./
COPY --chown=${HOST_USER_ID}:${HOST_GROUP_ID} bin/nix bin/nix

# Switch to user and build Nix environment
USER ${HOST_USERNAME}

# Pre-download Nix dependencies
RUN nix flake metadata --refresh \
    && nix build .#devcontainer --no-link

# Build the container environment and run setup
ENV DEFAULT_USER=${HOST_USERNAME}
RUN nix build .#devcontainer -o devcontainer-env && \
    sudo ./devcontainer-env/bin/setup-container && \
    sudo bash -c "echo '. /etc/profile.d/nix-env.sh' >> /etc/bash.bashrc" && \
    echo '. /etc/profile.d/nix-env.sh' >> ~/.bashrc && \
    # Fix all user directory permissions
    sudo chown -R ${HOST_USER_ID}:${HOST_GROUP_ID} /home/${HOST_USERNAME} || true

# Switch to root for system setup
USER root

# Docker-from-Docker entrypoint
RUN echo '#!/bin/bash' > /entrypoint.sh && \
    echo 'if [ -f /usr/local/share/docker-init.sh ]; then' >> /entrypoint.sh && \
    echo '    exec /usr/local/share/docker-init.sh "$@"' >> /entrypoint.sh && \
    echo 'else' >> /entrypoint.sh && \
    echo '    exec "$@"' >> /entrypoint.sh && \
    echo 'fi' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

# Clean up
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/* \
    && rm -f /home/${HOST_USERNAME}/nix_install.sh /home/${HOST_USERNAME}/nix_setup.sh

# Switch back to user
USER ${HOST_USERNAME}
WORKDIR /workspace

# Set entrypoint and default command
ENTRYPOINT ["/entrypoint.sh"]
CMD ["sleep", "infinity"]
