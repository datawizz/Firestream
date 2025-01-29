
# modules/common.nix
{ config, pkgs, lib, hostname, serviceAccountContent, ... }:

{
  imports = [
    ./startup-services-and-secrets.nix
  ];

  networking.hostName = hostname;
  networking.networkmanager.enable = true;

  # Users and groups
  users.groups = {
    serviceaccounts = {};
    app = {};
  };

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

  users.users.root.initialPassword = "trustme";

  # Service account handling
  system.activationScripts = {
    createCredentialFiles = {
      deps = [];
      text = ''
        mkdir -p /etc/gcp
        chmod 750 /etc/gcp
        chown root:serviceaccounts /etc/gcp
        ${if serviceAccountContent != "" then ''
          echo '${serviceAccountContent}' | base64 -d > /etc/gcp/service-account.json
          chmod 640 /etc/gcp/service-account.json
          chown root:serviceaccounts /etc/gcp/service-account.json
        '' else ""}
      '';
    };
  };

  environment.sessionVariables = {
    GOOGLE_APPLICATION_CREDENTIALS = "/etc/gcp/service-account.json";
  };

  # Common packages
  environment.systemPackages = with pkgs; [
    google-cloud-sdk
    jq
    tailscale
    cloudflared
    git
    gnumake
    docker-compose
    ctop
    btop
    kubectl
    helm
    k3s
  ];

  # Common services
  virtualisation.docker = {
    enable = true;
    enableOnBoot = true;
    daemon.settings.features.buildkit = true;
  };

  programs.gnupg.agent = {
    enable = true;
    enableSSHSupport = true;
  };

  services = {
    openssh = {
      enable = true;
      extraConfig = "StreamLocalBindUnlink yes";
    };

    startup-services-and-secrets.enable = true;

    tailscale = {
      enable = true;
      useRoutingFeatures = "both";
    };

    k3s = {
      enable = true;
      role = "server";
      tokenFile = "/var/lib/rancher/k3s/server/node-token";
      extraFlags = toString [
        "--disable-network-policy"
        "--flannel-backend=none"
        "--disable=traefik"
        "--cluster-cidr=10.244.0.0/16"
        "--service-cidr=10.245.0.0/16"
      ];
    };
  };

  # Kernel configuration
  boot = {
    kernelModules = [ "br_netfilter" "overlay" "bpf" "xdp_sockets" ];
    kernel.sysctl = {
      "net.bridge.bridge-nf-call-iptables" = 1;
      "net.bridge.bridge-nf-call-ip6tables" = 1;
      "net.ipv4.ip_forward" = 1;
      "net.ipv4.conf.all.forwarding" = 1;
      "net.ipv6.conf.all.forwarding" = 1;
      "kernel.unprivileged_bpf_disabled" = 0;
      "kernel.timer_migration" = 0;
    };
    kernelParams = [
      "security=apparmor"
      "systemd.unified_cgroup_hierarchy=1"
    ];
  };

  programs.nix-ld = {
    enable = true;
    libraries = with pkgs; [ zlib openssl ];
  };

  system.stateVersion = "24.11";
}
