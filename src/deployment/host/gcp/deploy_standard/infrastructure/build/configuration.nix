# default.nix
{ system ? builtins.currentSystem
, pkgs ? import <nixpkgs> { inherit system; }
}:

{ config ? {} }: # This accepts our runtime arguments

let
  # Convert the config into env vars
  makeEnvFile = ''
    echo "API_KEY=${config.apiKey or ""}" > /etc/app-env
    echo "DB_PASSWORD=${config.dbPassword or ""}" >> /etc/app-env
  '';

in {
  # Create a VM with our configuration
  vm = import "${pkgs.path}/nixos/lib/eval-config.nix" {
    inherit system;
    modules = [
      # Basic VM configuration
      "${pkgs.path}/nixos/modules/virtualisation/qemu-vm.nix"

      # Our custom configuration
      {
        # Basic system configuration
        system.stateVersion = "23.11";

        # Enable SSH for access
        services.openssh.enable = true;
        users.users.root.password = "root";

        # Create our environment file during system activation
        system.activationScripts.makeEnv = makeEnvFile;

        # Make VM have some reasonable memory
        virtualisation.memorySize = 2048;
        virtualisation.cores = 2;
      }
    ];
  };
}
