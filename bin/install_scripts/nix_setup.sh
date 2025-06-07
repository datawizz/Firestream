#!/bin/bash

## Setup the nix-shell in the container on a Debian host. Pre-cache packages defined in shell.nix file.

# Exit immediately if a command exits with a non-zero status
set -e

# Install Nix
bash /home/$USERNAME/nix_install.sh

# Configure Nix to change the download cache size
mkdir -p $HOME/.config/nix
if ! grep -q 'download-buffer-size' $HOME/.config/nix/nix.conf; then
  echo 'download-buffer-size = 500M' >> $HOME/.config/nix/nix.conf
fi


# Source the Nix environment
. $HOME/.nix-profile/etc/profile.d/nix.sh

# Add Nix to the user's PATH
export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH

# Ensure the Nix bin directories are in the PATH in .profile
if ! grep -q 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' $HOME/.profile; then
  echo 'export PATH=$HOME/.nix-profile/bin:$HOME/.nix-profile/sbin:$PATH' >> $HOME/.profile
fi


# Install the packages defined in the default.nix file
# Persist this in the container so that the packages are installed when the container is started
DEFAULT_NIX=/home/$USERNAME/default.nix
if [ -f "$DEFAULT_NIX" ]; then
  nix-env -f $DEFAULT_NIX -i
fi
