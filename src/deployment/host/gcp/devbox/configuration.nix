{ config, pkgs, lib, ... }:

{
  # ISO-specific settings
  # isoImage.contents = [
  #   {
  #     source = pkgs.memtest86plus + "/memtest.bin";
  #     target = "boot/memtest.bin";
  #   }
  # ];

  # Add these ISO-specific settings
  isoImage = {
    makeEfiBootable = true;
    makeUsbBootable = true;
    isoName = lib.mkForce "devbox.iso";
  };

  # Required kernel modules for live ISO
  boot = {
    # Use the latest Linux kernel
    kernelPackages = pkgs.linuxPackages_latest;

    # Required kernel modules for the live ISO
    initrd = {
      availableKernelModules = [
        "ata_piix" "uhci_hcd" "virtio_pci"
        "sr_mod" "virtio_blk" "ahci"
        "usbhid" "usb_storage" "sd_mod"
      ];
      # Add any additional kernel modules needed for hardware support
      kernelModules = [ ];
    };

    # Enable support for common filesystems
    supportedFilesystems = lib.mkForce [ "ext4" "btrfs" "xfs" "ntfs" "vfat" ];

    # Use systemd-boot as bootloader
    loader = {
      systemd-boot.enable = true;
      efi.canTouchEfiVariables = true;
    };
  };

    # Hostname Configuration
    networking = {
      hostName = "devbox";
      # Enable NetworkManager and disable wireless.enable to avoid conflicts
      networkmanager.enable = true;
      wireless.enable = false;

      useDHCP = lib.mkDefault true;
      firewall = {
        enable = true;
        allowedTCPPorts = [ 22 80 443 3000 8080 ];
      };
    };

  # Specify the host platform
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";

  # Define groups first
  users.groups.developer = {};
  users.groups.admin = {};

  # User Configuration
  users.users = {
    justin = {
      isNormalUser = true;
      group = "wheel";
      home = "/home/justin";
      # Set an empty password for live ISO
      initialPassword = "trustme";
      shell = pkgs.zsh;
      extraGroups = [ "wheel" "docker" "networkmanager" ];
      openssh.authorizedKeys.keys = [
        # Add your SSH public key here
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP..."
      ];
    };

    admin = {
      isNormalUser = false;
      isSystemUser = true;
      group = "admin";
      home = "/home/admin";
      # Set an empty password for live ISO
      initialPassword = "trustno1";
      shell = pkgs.zsh;
      extraGroups = [ "wheel" "docker" "networkmanager"];
    };
  };

  # System services configuration
  services = {
    # Enable OpenSSH server
    openssh = {
      enable = true;
      settings = {
        PermitRootLogin = lib.mkForce "no";
        PasswordAuthentication = false;
      };
    };

    # # Docker support
    # docker = {
    #   enable = true;
    #   autoPrune = {
    #     enable = true;
    #     dates = "weekly";
    #   };
    # };

    # Enable X11 with a basic desktop environment
    xserver = {
      enable = true;
      displayManager.gdm.enable = true;
      desktopManager.gnome.enable = true;
    };
  };

  # System features
  security = {
    sudo.wheelNeedsPassword = false;
    rtkit.enable = true;
  };

  # System Packages
  environment.systemPackages = with pkgs; [
    # Development tools
    gnumake
  ];

  # Locale and time settings
  time.timeZone = lib.mkDefault "UTC";
  i18n.defaultLocale = "en_US.UTF-8";
  console.keyMap = "us";


}
