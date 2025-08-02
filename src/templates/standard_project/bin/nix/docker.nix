# docker.nix - Docker-from-Docker support for development containers
{ pkgs, self ? null, system ? null }:

let
  # Configuration options
  config = {
    enableNonrootDocker = true;
    sourceSocket = "/var/run/docker-host.sock";
    targetSocket = "/var/run/docker.sock";
    installDockerBuildx = true;
  };

  # Docker packages
  dockerPackages = with pkgs; [
    docker-client
    docker-compose
    docker-buildx
    socat
  ];

  # Docker init script
  dockerInitScript = pkgs.writeScriptBin "docker-init.sh" ''
    #!/usr/bin/env bash
    set -e

    # Ensure socat is in PATH
    export PATH="${pkgs.socat}/bin:$PATH"

    SOCAT_PATH_BASE=/tmp/docker-from-docker
    SOCAT_LOG=''${SOCAT_PATH_BASE}.log
    SOCAT_PID=''${SOCAT_PATH_BASE}.pid

    # Wrapper function to only use sudo if not already root
    sudoIf() {
        if [ "$(id -u)" -ne 0 ]; then
            sudo "$@"
        else
            "$@"
        fi
    }

    # Log messages
    log() {
        echo -e "[$(date)] $@" | sudoIf tee -a ''${SOCAT_LOG} > /dev/null
    }

    # Determine the appropriate user
    USERNAME="$USER"
    if [ "$USERNAME" = "" ] || [ "$USERNAME" = "root" ]; then
        USERNAME=$(id -un)
    fi

    log "Docker-init starting for user ''${USERNAME}"

    # Setup docker group if it doesn't exist
    if ! grep -qE '^docker:' /etc/group; then
        log "Creating docker group..."
        sudoIf groupadd --system docker
    fi

    # Add user to docker group if not root
    if [ "''${USERNAME}" != "root" ]; then
        if ! groups ''${USERNAME} | grep -q docker; then
            log "Adding ''${USERNAME} to docker group"
            sudoIf usermod -aG docker "''${USERNAME}" || true
        fi
    fi

    # Main logic for socket access
    if [ "${config.sourceSocket}" != "${config.targetSocket}" ]; then
        # Check if source socket exists
        if [ ! -S "${config.sourceSocket}" ]; then
            log "ERROR: Source socket ${config.sourceSocket} does not exist"
            exit 1
        fi

        # Check if we can access the source socket
        if sudoIf -u ''${USERNAME} test -r "${config.sourceSocket}" 2>/dev/null && \
           sudoIf -u ''${USERNAME} test -w "${config.sourceSocket}" 2>/dev/null; then
            # User can access source socket directly, just create symlink
            log "Creating symlink from ${config.targetSocket} to ${config.sourceSocket}"
            sudoIf rm -f "${config.targetSocket}"
            sudoIf ln -sf "${config.sourceSocket}" "${config.targetSocket}"
        else
            # User cannot access source socket, need socat proxy
            log "Starting socat proxy"

            # Kill any existing socat process
            if [ -f "''${SOCAT_PID}" ]; then
                OLD_PID=$(cat ''${SOCAT_PID} 2>/dev/null || echo "0")
                if [ "''${OLD_PID}" != "0" ] && ps -p ''${OLD_PID} > /dev/null 2>&1; then
                    sudoIf kill ''${OLD_PID} || true
                fi
            fi

            # Remove existing target socket
            sudoIf rm -f "${config.targetSocket}"

            # Start socat proxy
            (sudoIf ${pkgs.socat}/bin/socat \
                UNIX-LISTEN:${config.targetSocket},fork,mode=660,user=''${USERNAME},group=docker \
                UNIX-CONNECT:${config.sourceSocket} \
                2>&1 | sudoIf tee -a ''${SOCAT_LOG} > /dev/null) &
            SOCAT_NEW_PID=$!
            echo "''${SOCAT_NEW_PID}" | sudoIf tee ''${SOCAT_PID} > /dev/null

            sleep 1
            log "Socat proxy started with PID ''${SOCAT_NEW_PID}"
        fi
    fi

    log "Docker-init completed"
    exec "$@"
  '';

  # Environment variables for Docker
  envVars = ''
    # Docker environment variables
    export DOCKER_HOST=unix://${config.targetSocket}
    export DOCKER_BUILDKIT=1

    # Add docker plugins to PATH if buildx is installed
    ${if config.installDockerBuildx then ''
      export DOCKER_CLI_PLUGINS_PATH="$HOME/.docker/cli-plugins"
      mkdir -p "$DOCKER_CLI_PLUGINS_PATH" 2>/dev/null || true
      if [ ! -f "$DOCKER_CLI_PLUGINS_PATH/docker-buildx" ]; then
        ln -sf ${pkgs.docker-buildx}/bin/docker-buildx "$DOCKER_CLI_PLUGINS_PATH/docker-buildx" 2>/dev/null || true
      fi
    '' else ""}
  '';

  # Profile script
  profileScript = ''
    ${envVars}

    # Initialize Docker socket proxy if needed
    if [ -f /usr/local/share/docker-init.sh ]; then
      # Check if we need to run docker-init
      if [ -S ${config.sourceSocket} ] && [ ! -S ${config.targetSocket} ]; then
        /usr/local/share/docker-init.sh true 2>/dev/null || true
      fi
    fi
  '';

  # Setup script
  setupScript = pkgs.writeScript "setup-docker" ''
    #!${pkgs.bash}/bin/bash
    set -e

    echo "Setting up Docker-from-Docker environment..."

    # Create necessary directories
    mkdir -p /usr/local/share
    mkdir -p "$HOME/.docker/cli-plugins"

    # Copy docker-init.sh to system location
    if [ -f ${dockerInitScript}/bin/docker-init.sh ]; then
        cp -f ${dockerInitScript}/bin/docker-init.sh /usr/local/share/docker-init.sh
        chmod 755 /usr/local/share/docker-init.sh 2>/dev/null || true
    fi

    echo "Docker-from-Docker setup complete!"
  '';

in
{
  # Packages to be included
  packages = dockerPackages;

  # Shell hook for development
  shellHook = ''
    ${envVars}
    echo "Docker-from-Docker environment loaded"
  '';

  # Profile script for persistent environment
  profileScript = profileScript;

  # Setup script for initialization
  setupScript = setupScript;

  # Apps for Docker module
  apps = {
    docker-init = {
      type = "app";
      program = "${dockerInitScript}/bin/docker-init.sh";
    };
  };
}
