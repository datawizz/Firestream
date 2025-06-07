# Docker-from-Docker Nix Module
# Based on Microsoft's VS Code dev containers script
{ pkgs }:

let
  # Configuration options with defaults matching VS Code devcontainer defaults
  config = {
    # [Option] Engine/CLI Version
    dockerVersion = "latest";
    # [Option] Use the OSS Moby Engine instead of the licensed Docker Engine
    useMoby = true;
    # [Option] Enable non-root Docker access in container
    enableNonrootDocker = true;
    # Docker Compose version
    dockerComposeVersion = "v2";
    # Socket configuration
    sourceSocket = "/var/run/docker-host.sock";
    targetSocket = "/var/run/docker.sock";
    # User configuration
    username = "automatic";
    # Install Docker Buildx plugin
    installDockerBuildx = true;
  };

  # Docker packages based on configuration
  dockerPkg = if config.useMoby then pkgs.docker else pkgs.docker;

  # Docker init script that handles socket proxying and permissions
  dockerInitScript = pkgs.writeScriptBin "docker-init.sh" ''
    #!/usr/bin/env bash
    #-------------------------------------------------------------------------------------------------------------
    # Copyright (c) Microsoft Corporation. All rights reserved.
    # Licensed under the MIT License. See https://go.microsoft.com/fwlink/?linkid=2090316 for license information.
    #-------------------------------------------------------------------------------------------------------------

    set -e

    # Ensure socat is in PATH
    export PATH="${pkgs.socat}/bin:$PATH"

    SOCAT_PATH_BASE=/tmp/vscr-docker-from-docker
    SOCAT_LOG=''${SOCAT_PATH_BASE}.log
    SOCAT_PID=''${SOCAT_PATH_BASE}.pid

    # Wrapper function to only use sudo if not already root
    sudoIf()
    {
        if [ "$(id -u)" -ne 0 ]; then
            sudo "$@"
        else
            "$@"
        fi
    }

    # Log messages
    log()
    {
        echo -e "[$(date)] $@" | sudoIf tee -a ''${SOCAT_LOG} > /dev/null
    }

    # Determine the appropriate non-root user
    determine_user() {
        local USERNAME="${config.username}"

        if [ "''${USERNAME}" = "auto" ] || [ "''${USERNAME}" = "automatic" ]; then
            USERNAME=""
            POSSIBLE_USERS=("vscode" "node" "codespace" "firestream" "$(awk -v val=1000 -F ":" '$3==val{print $1}' /etc/passwd)")
            for CURRENT_USER in "''${POSSIBLE_USERS[@]}"; do
                if id -u ''${CURRENT_USER} > /dev/null 2>&1; then
                    USERNAME=''${CURRENT_USER}
                    break
                fi
            done
            if [ "''${USERNAME}" = "" ]; then
                USERNAME=root
            fi
        elif [ "''${USERNAME}" = "none" ] || ! id -u ''${USERNAME} > /dev/null 2>&1; then
            USERNAME=root
        fi

        echo "''${USERNAME}"
    }

    USERNAME=$(determine_user)

    echo -e "\n** $(date) **" | sudoIf tee -a ''${SOCAT_LOG} > /dev/null
    log "Docker-init starting for user ''${USERNAME}"
    log "Source socket: ${config.sourceSocket}"
    log "Target socket: ${config.targetSocket}"

    # Setup docker group if it doesn't exist
    if ! grep -qE '^docker:' /etc/group; then
        log "Creating missing docker group..."
        sudoIf groupadd --system docker
    fi

    # Add user to docker group if not root
    if [ "''${USERNAME}" != "root" ] && id -u ''${USERNAME} > /dev/null 2>&1; then
        if ! groups ''${USERNAME} | grep -q docker; then
            log "Adding ''${USERNAME} to docker group"
            sudoIf usermod -aG docker "''${USERNAME}" || true
        fi
    fi

    # Main logic for socket access
    if [ "${toString config.enableNonrootDocker}" = "1" ] && [ "''${USERNAME}" != "root" ]; then
        log "Non-root Docker enabled for user ''${USERNAME}"

        # Check if source socket exists
        if [ ! -S "${config.sourceSocket}" ]; then
            log "ERROR: Source socket ${config.sourceSocket} does not exist or is not a socket"
            exit 1
        fi

        # Get socket ownership info
        SOCKET_OWNER_UID=$(stat -c '%u' ${config.sourceSocket} 2>/dev/null || echo "0")
        SOCKET_OWNER_GID=$(stat -c '%g' ${config.sourceSocket} 2>/dev/null || echo "0")
        DOCKER_GID=$(getent group docker | cut -d: -f3)
        USER_GROUPS=$(id -G ''${USERNAME})

        log "Socket owner: UID=''${SOCKET_OWNER_UID}, GID=''${SOCKET_OWNER_GID}"
        log "Docker group GID: ''${DOCKER_GID}"
        log "User ''${USERNAME} groups: ''${USER_GROUPS}"

        # Check if user can already access the socket
        CAN_ACCESS=false
        if sudoIf -u ''${USERNAME} test -r "${config.sourceSocket}" 2>/dev/null && \
           sudoIf -u ''${USERNAME} test -w "${config.sourceSocket}" 2>/dev/null; then
            CAN_ACCESS=true
            log "User ''${USERNAME} already has direct access to ${config.sourceSocket}"
        fi

        # If we need different source and target sockets
        if [ "${config.sourceSocket}" != "${config.targetSocket}" ]; then
            if [ "''${CAN_ACCESS}" = "true" ]; then
                # User can access source socket directly, just create symlink
                log "Creating symlink from ${config.targetSocket} to ${config.sourceSocket}"
                sudoIf rm -f "${config.targetSocket}"
                sudoIf ln -sf "${config.sourceSocket}" "${config.targetSocket}"
            else
                # User cannot access source socket, need socat proxy
                log "User cannot access source socket directly, starting socat proxy"

                # Kill any existing socat process
                if [ -f "''${SOCAT_PID}" ]; then
                    OLD_PID=$(cat ''${SOCAT_PID} 2>/dev/null || echo "0")
                    if [ "''${OLD_PID}" != "0" ] && ps -p ''${OLD_PID} > /dev/null 2>&1; then
                        log "Killing existing socat process ''${OLD_PID}"
                        sudoIf kill ''${OLD_PID} || true
                    fi
                fi

                # Remove existing target socket
                sudoIf rm -f "${config.targetSocket}"

                # Start socat proxy
                log "Starting socat proxy: ${config.sourceSocket} -> ${config.targetSocket}"
                (sudoIf ${pkgs.socat}/bin/socat \
                    UNIX-LISTEN:${config.targetSocket},fork,mode=660,user=''${USERNAME},group=docker \
                    UNIX-CONNECT:${config.sourceSocket} \
                    2>&1 | sudoIf tee -a ''${SOCAT_LOG} > /dev/null) &
                SOCAT_NEW_PID=$!
                echo "''${SOCAT_NEW_PID}" | sudoIf tee ''${SOCAT_PID} > /dev/null

                # Wait a moment for socat to start
                sleep 1

                # Verify socat started successfully
                if ps -p ''${SOCAT_NEW_PID} > /dev/null 2>&1; then
                    log "Socat proxy started successfully with PID ''${SOCAT_NEW_PID}"

                    # Verify the target socket was created
                    if [ -S "${config.targetSocket}" ]; then
                        log "Target socket ${config.targetSocket} created successfully"
                        ls -la "${config.targetSocket}" 2>&1 | sudoIf tee -a ''${SOCAT_LOG}
                    else
                        log "ERROR: Target socket ${config.targetSocket} was not created"
                    fi
                else
                    log "ERROR: Socat process failed to start"
                    exit 1
                fi
            fi
        else
            # Source and target are the same
            if [ "''${CAN_ACCESS}" != "true" ]; then
                log "WARNING: User cannot access ${config.sourceSocket} and source/target are the same"
                log "This configuration cannot work without changing socket permissions"
            fi
        fi
    else
        log "Running as root or non-root Docker disabled, no proxy needed"
        if [ "${config.sourceSocket}" != "${config.targetSocket}" ] && [ ! -e "${config.targetSocket}" ]; then
            sudoIf ln -sf "${config.sourceSocket}" "${config.targetSocket}"
        fi
    fi

    log "Docker-init completed"

    # Execute whatever commands were passed in (if any). This allows us
    # to set this script to ENTRYPOINT while still executing the default CMD.
    set +e
    exec "$@"
  '';

  # Container setup script for Docker
  setupDockerScript = pkgs.writeScriptBin "setup-docker-from-docker" ''
    #!/usr/bin/env bash
    set -e

    echo "Setting up Docker-from-Docker environment..."

    # Create necessary directories
    mkdir -p /usr/local/share
    mkdir -p /etc/profile.d

    # Copy docker-init.sh to system location with proper permissions
    if [ -f ${dockerInitScript}/bin/docker-init.sh ]; then
        # Use -p to preserve permissions from source
        cp -pf ${dockerInitScript}/bin/docker-init.sh /usr/local/share/docker-init.sh
        # Try to ensure it's executable, but don't fail if we can't
        chmod 755 /usr/local/share/docker-init.sh 2>/dev/null || true
    else
        echo "ERROR: docker-init.sh not found in Nix store!"
        exit 1
    fi

    # Create profile script for automatic initialization
    cat > /etc/profile.d/docker-from-docker.sh << 'EOF'
    # Docker-from-Docker initialization
    if [ -f /usr/local/share/docker-init.sh ]; then
        # Check if we need to run docker-init
        NEED_INIT=false

        # Check if source socket exists
        if [ -S ${config.sourceSocket} ]; then
            # Check if we can access it
            if ! test -r ${config.sourceSocket} 2>/dev/null || ! test -w ${config.sourceSocket} 2>/dev/null; then
                # Cannot access source socket
                if [ "${config.sourceSocket}" != "${config.targetSocket}" ]; then
                    # Check if target socket exists and is accessible
                    if [ ! -S ${config.targetSocket} ] || ! test -r ${config.targetSocket} 2>/dev/null || ! test -w ${config.targetSocket} 2>/dev/null; then
                        NEED_INIT=true
                    fi
                else
                    # Source and target are the same, nothing we can do
                    NEED_INIT=false
                fi
            fi
        fi

        if [ "$NEED_INIT" = "true" ]; then
            # Run docker-init in background to setup socket proxy if needed
            /usr/local/share/docker-init.sh true 2>/dev/null || true
        fi
    fi

    # Add docker completion if available
    if [ -f /etc/bash_completion.d/docker ]; then
        . /etc/bash_completion.d/docker
    fi

    # Enable Docker BuildKit by default
    export DOCKER_BUILDKIT=1
    EOF

    chmod 644 /etc/profile.d/docker-from-docker.sh

    echo "Docker-from-Docker setup complete!"
    echo "Run 'source /etc/profile.d/docker-from-docker.sh' to initialize in current shell"
  '';

in
{
  # List of packages to install
  packages = with pkgs; [
    # Docker CLI and related tools
    docker-client

    # Docker Compose based on configuration
    (if config.dockerComposeVersion == "v1" then docker-compose
     else if config.dockerComposeVersion == "v2" then docker-compose
     else null)

    # Docker Buildx if enabled
    (if config.installDockerBuildx then docker-buildx else null)

    # Socket proxy for non-root access
    socat

    # Additional utilities
    gnupg
    cacert
  ] ++ (builtins.filter (x: x != null) []);

  # Scripts to be made available
  scripts = [
    dockerInitScript
    setupDockerScript
  ];

  # Named script exports for easier access
  dockerInit = dockerInitScript;
  setupDocker = setupDockerScript;

  # Environment setup for profile.d
  profileScript = ''
    # Docker-from-Docker environment variables
    export DOCKER_HOST=unix://${config.targetSocket}

    # Enable new "BUILDKIT" mode for Docker CLI (set permanently)
    export DOCKER_BUILDKIT=1

    # Add docker plugins to PATH if buildx is installed
    ${if config.installDockerBuildx then ''
      export DOCKER_CLI_PLUGINS_PATH="$HOME/.docker/cli-plugins"
      mkdir -p "$DOCKER_CLI_PLUGINS_PATH"
      if [ ! -f "$DOCKER_CLI_PLUGINS_PATH/docker-buildx" ]; then
        ln -sf ${pkgs.docker-buildx}/bin/docker-buildx "$DOCKER_CLI_PLUGINS_PATH/docker-buildx"
      fi
    '' else ""}

    # Initialize Docker socket proxy if needed
    if [ -f /usr/local/share/docker-init.sh ]; then
      # Check if we need to run docker-init
      NEED_INIT=false

      # Check if source socket exists
      if [ -S ${config.sourceSocket} ]; then
        # Check if we can access it
        if ! test -r ${config.sourceSocket} 2>/dev/null || ! test -w ${config.sourceSocket} 2>/dev/null; then
          # Cannot access source socket
          if [ "${config.sourceSocket}" != "${config.targetSocket}" ]; then
            # Check if target socket exists and is accessible
            if [ ! -S ${config.targetSocket} ] || ! test -r ${config.targetSocket} 2>/dev/null || ! test -w ${config.targetSocket} 2>/dev/null; then
              NEED_INIT=true
            fi
          fi
        fi
      fi

      if [ "$NEED_INIT" = "true" ]; then
        /usr/local/share/docker-init.sh true 2>/dev/null || true
      fi
    fi
  '';

  # Export configuration for reference
  inherit config;
}
