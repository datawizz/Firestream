{ config, pkgs, ... }:

{
  # Provide a QCOW2 builder in the system config
  system.build.qcow2 = import <nixpkgs/nixos/lib/make-disk-image.nix> {
    pkgs  = pkgs;
    lib   = pkgs.lib;
    config = config;
    format = "qcow2";
  };

  # Preserve your filesystem settings
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };

  # Preserve your boot settings
  boot = {
    # Let NixOS resize partitions automatically
    growPartition = true;

    # Ensure messages appear on the QEMU console
    kernelParams = [ "console=ttyS0" ];

    # Configure GRUB to install on /dev/vda
    loader = {
      grub.device = "/dev/vda";
      timeout = 0;
    };
  };
}
