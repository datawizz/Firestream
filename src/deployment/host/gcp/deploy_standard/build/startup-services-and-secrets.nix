

{ config, lib, pkgs, ... }:
with lib;
let
  cfg = config.services.startup-services-and-secrets;
in {
  options.services.startup-services-and-secrets = {
    enable = mkEnableOption "Startup Services and Secrets";
    secretsPath = mkOption {
      type = types.str;
      default = "/home/app/secrets.json";
      description = "Path to the secrets JSON file";
    };
  };

  config = mkIf cfg.enable {
    # Secret placement service
    systemd.services.place-secrets = {
      description = "Load secrets from Google Cloud and place them";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      before = [ "cloudflared.service" "tailscale-up.service" ];
      path = with pkgs; [ google-cloud-sdk jq coreutils ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        User = "root";
        Group = "root";
        Environment = [
          "GOOGLE_APPLICATION_CREDENTIALS=/etc/gcp/service-account.json"
        ];
        ExecStart = pkgs.writeShellScript "place-secrets" ''
          set -e -o pipefail

          # Authenticate with GCP
          gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS

          # Fetch and save secrets
          gcloud secrets versions access latest --secret deployment-secrets > ${cfg.secretsPath}

          # Create required directories
          mkdir -p /home/app /etc/cloudflared

          # Place Cloudflare credentials
          jq -r '.tunnel.credentials_b64' ${cfg.secretsPath} | base64 -d > /home/app/credentials.json
          jq -r '.tunnel.yaml_config_b64' ${cfg.secretsPath} | base64 -d > /etc/cloudflared/config.yaml

          # Place Tailscale authkey
          jq -r '.tailscale.key' ${cfg.secretsPath} > /home/app/tailscale_authkey

          # Set proper permissions
          chown -R app:app /home/app
          chmod 600 /home/app/credentials.json /home/app/tailscale_authkey
          chmod 644 /etc/cloudflared/config.yaml
        '';
      };
    };

    # Tailscale startup service
    systemd.services.tailscale-up = {
      description = "Start Tailscale with authentication";
      after = [ "place-secrets.service" "tailscaled.service" ];
      requires = [ "place-secrets.service" "tailscaled.service" ];
      wantedBy = [ "multi-user.target" ];
      path = [ pkgs.tailscale ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        User = "root";
        Group = "root";
        ExecStart = pkgs.writeShellScript "start-tailscale" ''
          set -e -o pipefail
          tailscale up --auth-key=file:/home/app/tailscale_authkey --ssh
        '';
      };
    };


    # Cloudflared service
    systemd.services.cloudflared = {
      description = "Cloudflare Tunnel daemon";
      after = [ "place-secrets.service" ];
      requires = [ "place-secrets.service" ];
      wantedBy = [ "multi-user.target" ];
      path = [ pkgs.cloudflared ];
      serviceConfig = {
        Type = "simple";
        User = "app";
        Group = "app";
        ExecStart = "${pkgs.cloudflared}/bin/cloudflared tunnel run --credentials-file=/home/app/credentials.json";
        Restart = "always";
        RestartSec = "10";
      };
    };

    # Enable base tailscale service
    services.tailscale = {
      enable = true;
      useRoutingFeatures = "both";
    };
  };
}
