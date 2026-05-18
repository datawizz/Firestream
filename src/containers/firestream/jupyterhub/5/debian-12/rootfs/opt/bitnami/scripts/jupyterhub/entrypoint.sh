#!/bin/bash
# Copyright Firestream. MIT License.
# JupyterHub container entrypoint
#
# shellcheck disable=SC1091

set -o errexit
set -o nounset
set -o pipefail
# set -o xtrace # Uncomment for debugging

# Load JupyterHub environment
. /opt/bitnami/scripts/jupyterhub-env.sh

# Load libraries
. /opt/bitnami/scripts/libjupyterhub.sh

print_welcome_page

# Only run setup if we're starting the main service
if [[ "$1" = "/opt/bitnami/scripts/jupyterhub/run.sh" ]]; then
    # Source environment defaults
    . /opt/bitnami/scripts/jupyterhub/env-defaults.sh

    # Load secrets from _FILE environment variables
    . /opt/bitnami/scripts/jupyterhub/secrets.sh

    # Validate environment
    . /opt/bitnami/scripts/jupyterhub/validate.sh

    # Run configuration
    . /opt/bitnami/scripts/jupyterhub/config.sh

    # Initialize
    . /opt/bitnami/scripts/jupyterhub/init.sh

    # Run post-init scripts
    /post-init.sh

    info "** JupyterHub setup finished! **"
fi

echo ""
exec "$@"
