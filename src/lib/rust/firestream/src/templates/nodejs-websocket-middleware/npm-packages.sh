#!/bin/bash

# Location to store the downloaded packages
pack_destination="/workspace/npm/packages"

pack_source="/workspace/services/javascript/websocket_middleware/package.json"

# Create the destination directory if it doesn't exist
mkdir -p "$pack_destination"

# Read the package.json
packages=$(jq -r 'keys[] as $k | "\($k)-\(.[$k].version)"' < $pack_source)



# Loop through the packages and run npm pack for each
for package in $packages; do
  npm pack "$package" --no-save --no-bin-links --prefix "$pack_destination"
done
