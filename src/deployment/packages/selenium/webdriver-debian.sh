#!/bin/bash 
set -e # Exit immediately if a command exits with a non-zero status.

# Webdriver Installation

# The webdriver is required for rendering pages using Selenium. 
# This allows dynamic content which is rendered by Javascript to be captured by the crawler.


# Set the desired location for the WebDriver
HOST_PATH=${1:-"/usr/local/bin"}

VERSION=$(curl -s https://chromedriver.storage.googleapis.com/LATEST_RELEASE)

# Check the system's architecture
ARCHITECTURE=$(uname -m)
FILE_NAME=""
case "$ARCHITECTURE" in
  x86_64)
    FILE_NAME="chromedriver_linux64.zip"
    ;;
  arm*|aarch64*)
    FILE_NAME="chromedriver_linux64_arm.zip"
    ;;
  *)
    echo "Unsupported architecture: $ARCHITECTURE"
    exit 1
    ;;
esac

# Download and extract the latest Chrome WebDriver
echo "Downloading Chrome WebDriver for $ARCHITECTURE..."
wget -q --show-progress -O /tmp/$FILE_NAME "https://chromedriver.storage.googleapis.com/$VERSION/$FILE_NAME"

echo "Extracting Chrome WebDriver..."
mkdir -p "$HOST_PATH"
unzip -o /tmp/$FILE_NAME -d "$HOST_PATH"
rm /tmp/$FILE_NAME
chmod +x "$HOST_PATH/chromedriver"

# Add the WebDriver's location to the PATH, if it's not already there
if ! grep -q "$HOST_PATH/chromedriver" "$HOME/.bashrc"; then
  echo "Adding Chrome WebDriver to PATH..."
  echo "export PATH=\$PATH:$HOST_PATH" >> "$HOME/.bashrc"

  echo "Chrome WebDriver added to PATH."
else
  echo "Chrome WebDriver is already in the PATH."
fi

echo "Done."
