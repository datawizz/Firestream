# QEMU guest configuration
{ config, pkgs, lib, modulesPath, system, serviceAccountFile ? "", hostname ? "prodbox", ... }:
{
  imports = [
    # Include the QEMU guest profile kernal extensions
    (modulesPath + "/profiles/qemu-guest.nix")
    ./startup-services-and-secrets.nix
  ];


  # Your existing configuration remains the same
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };

  boot.loader.grub.enable = true;
  boot.loader.grub.device = "/dev/sda";
  boot.loader.grub.useOSProber = true;

  networking.hostName = "prodbox";
  networking.networkmanager.enable = true;

  # Create a group for managing service account access
  users.groups.serviceaccounts = {};
  users.groups.app = {};

  users.users.app = {
    isNormalUser = true;
    group = "app";
    description = "app user is non-root, non-sudoer, user for running applications";
    extraGroups = [ "networkmanager" "docker" "serviceaccounts" "ssh-agent-users"];
    shell = pkgs.bash;
    packages = with pkgs; [];
    initialPassword = "trustme";
    home = "/home/app";
    createHome = true;
  };

  # TODO this is only here for development
  # setup password for root user
  users.users.root = {
    initialPassword = "trustme";
  };

  # system.activationScripts = {
  #   createCredentialFiles = {
  #     deps = [];
  #     text = ''
  #       # Create directory with proper permissions
  #       mkdir -p /etc/gcp
  #       # Set directory permissions to allow group access
  #       chmod 750 /etc/gcp
  #       # Set directory group ownership
  #       chown root:serviceaccounts /etc/gcp
  #       # Write service account credentials if provided, decoding from base64
  #       ${if serviceAccountContent != "" then ''
  #         echo '${serviceAccountContent}' | base64 -d > /etc/gcp/service-account.json
  #         chmod 640 /etc/gcp/service-account.json
  #         chown root:serviceaccounts /etc/gcp/service-account.json
  #       '' else ""}
  #     '';
  #   };
  # };

  # Set GOOGLE_APPLICATION_CREDENTIALS for all users system-wide
  environment.sessionVariables = {
    GOOGLE_APPLICATION_CREDENTIALS = "/etc/gcp/service-account.json";
  };


  # Rest of your configuration remains the same
  environment.systemPackages = with pkgs; [
    # Used for access to Google Cloud Platform resources
    google-cloud-sdk
    jq
    tailscale
    cloudflared
    git
    gnumake
    docker-compose
    ctop
    btop
    kubectl # Added for k3s management
    helm
    k3s # Added for k3s CLI tools
  ];

  virtualisation.docker = {
    enable = true;
    enableOnBoot = true;
    daemon.settings = {
      features = {
        buildkit = true;
      };
    };
  };

  programs.gnupg.agent = {
    enable = true;
    enableSSHSupport = true;
  };

  services.openssh = {
    enable = true;
    extraConfig = ''
      StreamLocalBindUnlink yes
    '';
  };

  services.startup-services-and-secrets.enable = true;

  services.tailscale = {
    enable = true;
    useRoutingFeatures = "both";
  };

  services.k3s = {
      enable = true;
      role = "server";
      tokenFile = "/var/lib/rancher/k3s/server/node-token";
      extraFlags = toString [
        # Disable default CNI and networking
        "--disable-network-policy"
        "--flannel-backend=none"
        "--disable=traefik"  # Optional: disable traefik if you plan to use a different ingress
        # Configure cluster for Cilium
        "--cluster-cidr=10.244.0.0/16"  # Pod CIDR
        "--service-cidr=10.245.0.0/16"  # Service CIDR
      ];
    };


  # Enhanced kernel configuration for Cilium
  boot = {
    kernelModules = [
      "br_netfilter"
      "overlay"
      "bpf"
      "xdp_sockets"  # For XDP support
    ];
    kernel.sysctl = {
      "net.bridge.bridge-nf-call-iptables" = 1;
      "net.bridge.bridge-nf-call-ip6tables" = 1;
      "net.ipv4.ip_forward" = 1;
      "net.ipv4.conf.all.forwarding" = 1;
      "net.ipv6.conf.all.forwarding" = 1;
      # Cilium-specific sysctls
      "kernel.unprivileged_bpf_disabled" = 0;
      "kernel.timer_migration" = 0;
    };
    kernelParams = [
      "security=apparmor"  # If using AppArmor
      "systemd.unified_cgroup_hierarchy=1"  # Required for BPF-based features
    ];
  };

  programs.nix-ld = {
    enable = true;
    libraries = with pkgs; [
      zlib
      openssl
    ];
  };

  system.stateVersion = "24.11";
}
