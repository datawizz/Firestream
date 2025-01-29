#!/bin/bash

# Exit on error
set -e

# Define directories
BUILD_DIR="/workspace/_build"
CURRENT_DIR=$(pwd)
TEMP_DIR=$(mktemp -d)

# Create build directory if it doesn't exist
mkdir -p "$BUILD_DIR"

# Create machine-config.nix
cat > "$TEMP_DIR/machine-config.nix" << 'EOF'
{ pkgs, lib, ... }:

with lib;

{
  imports = [
    <nixpkgs/nixos/modules/profiles/qemu-guest.nix>
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

    users.extraUsers.root.password = "";
  };
}
EOF

# Create build-qcow2.nix
cat > "$TEMP_DIR/build-qcow2.nix" << 'EOF'
{ config, lib, pkgs, ... }:

with lib;

{
  imports = [
    <nixpkgs/nixos/modules/installer/cd-dvd/channel.nix>
    ./machine-config.nix
  ];

  system.build.qcow2 = import <nixpkgs/nixos/lib/make-disk-image.nix> {
    inherit lib config;
    pkgs = import <nixpkgs> { inherit (pkgs) system; };
    diskSize = 8192;
    format = "qcow2";
    configFile = pkgs.writeText "configuration.nix" (pkgs.lib.readFile ./machine-config.nix);
  };
}
EOF

echo "Building NixOS QCOW2 image..."
cd "$TEMP_DIR"

# Build the image
RESULT_PATH=$(nix-build '<nixpkgs/nixos>' \
    -A config.system.build.qcow2 \
    --arg configuration "{ imports = [ ./build-qcow2.nix ]; }" \
    --no-out-link)

echo "Build completed at: $RESULT_PATH"

# Find the QCOW2 file
QCOW2_FILE=$(find "$RESULT_PATH" -name "*.qcow2" 2>/dev/null)

if [ -n "$QCOW2_FILE" ] && [ -f "$QCOW2_FILE" ]; then
    echo "Found QCOW2 image at: $QCOW2_FILE"
    cp "$QCOW2_FILE" "$BUILD_DIR/nixos-vm.qcow2"
    echo "Successfully copied VM image to $BUILD_DIR/nixos-vm.qcow2"
    du -h "$BUILD_DIR/nixos-vm.qcow2"
else
    echo "Error: Could not find QCOW2 image in $RESULT_PATH"
    echo "Contents of result directory:"
    ls -la "$RESULT_PATH"
    exit 1
fi

# Cleanup
cd "$CURRENT_DIR"
rm -rf "$TEMP_DIR"


nix-build '<nixpkgs/nixos>' -A config.system.build.qcow2 --arg configuration "{ imports = [ ./build-qcow2.nix ]; }"
