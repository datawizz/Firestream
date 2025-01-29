{ config, pkgs, lib, ... }:
with lib;
{
  imports = [
    <nixpkgs/nixos/modules/profiles/qemu-guest.nix>
    <nixpkgs/nixos/modules/installer/cd-dvd/channel.nix>
  ];

  config = {
    fileSystems."/" = {
      device = "/dev/disk/by-label/nixos";
      fsType = "ext4";
      autoResize = true;
    };

    boot.growPartition = true;
    boot.kernelParams = [ "console=ttyS0" ];
    boot.loader.grub.device = "/dev/vda";
    boot.loader.timeout = 0;

    users.users.root.password = "";

    system.build.qcow2 = import <nixpkgs/nixos/lib/make-disk-image.nix> {
      inherit pkgs lib config;
      diskSize = 8192;
      format = "qcow2";
      configFile = pkgs.writeText "configuration.nix" ''
        { config, pkgs, ... }: {
          imports = [ ];

          services.openssh.enable = true;
        }
      '';
    };
  };
}
