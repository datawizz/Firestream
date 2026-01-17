#!/bin/bash

## Setup the nix-shell in the container on a Debian host

# Exit immediately if a command exits with a non-zero status
set -e

# Get USERNAME from environment or default
USERNAME="${USERNAME:-developer}"

# Install Nix
bash /home/$USERNAME/nix_install.sh --no-daemon

# Configure Nix to change the download cache size
mkdir -p $HOME/.config/nix
if ! grep -q 'download-buffer-size' $HOME/.config/nix/nix.conf; then
  echo 'download-buffer-size = 500M' >> $HOME/.config/nix/nix.conf
fi

# Add experimental features for flakes
if ! grep -q 'experimental-features' $HOME/.config/nix/nix.conf; then
  echo 'experimental-features = nix-command flakes' >> $HOME/.config/nix/nix.conf
fi

# Source the Nix environment
. $HOME/.nix-profile/etc/profile.d/nix.sh

# Add Nix to the user's PATH
export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH

# Ensure the Nix bin directories are in the PATH in .profile
if ! grep -q 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' $HOME/.profile; then
  echo 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' >> $HOME/.profile
fi

# Also add to .bashrc for non-login shells
if ! grep -q 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' $HOME/.bashrc; then
  echo 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' >> $HOME/.bashrc
fi

echo "Nix installation completed successfully"
