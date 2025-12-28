#!/bin/bash
# Copyright Firestream. Apache-2.0 License.
# JupyterHub initial setup
#
# shellcheck disable=SC1090,SC1091

set -o errexit
set -o nounset
set -o pipefail
# set -o xtrace # Uncomment for debugging

# Load JupyterHub environment
. /opt/bitnami/scripts/jupyterhub-env.sh

# Load libraries
. /opt/bitnami/scripts/libjupyterhub.sh

info "Setting up JupyterHub..."

# Ensure JupyterHub environment variables are valid
jupyterhub_validate

# Ensure JupyterHub is initialized
jupyterhub_initialize

info "JupyterHub setup complete"
