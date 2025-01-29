#!/bin/bash
set -euo pipefail

function check_openssl_installed() {
    if ! command -v openssl &> /dev/null
    then
        echo "openssl is not installed"
        exit 1
    fi
}

function generate_certificate() {
    mkdir -p $HOME/certs
    openssl req \
        -new \
        -newkey rsa:4096 \
        -days 365 \
        -nodes \
        -x509 \
        -subj "/emailAddress=$EMAIL_ADDRESS/C=$COUNTRY_NAME/ST=$STATE_NAME/L=$LOCALITY_NAME/O=$ORGANIZATION_NAME/OU=$ORGANIZATIONAL_UNIT_NAME/CN=$COMMON_NAME" \
        -keyout $HOME/certs/server.key \
        -out $HOME/certs/server.crt \
        || { echo 'Failed to generate certificate'; exit 1; }
}

COUNTRY_NAME=${COUNTRY_NAME:="US"}
STATE_NAME=${STATE_NAME:="New Mexico"}
LOCALITY_NAME=${LOCALITY_NAME:="Roswell"}
ORGANIZATION_NAME=${ORGANIZATION_NAME:="ACME Co, LLC."}
ORGANIZATIONAL_UNIT_NAME=${ORGANIZATIONAL_UNIT_NAME:="ACME Department"}
COMMON_NAME=${COMMON_NAME:="www.domain.com"}
EMAIL_ADDRESS=${EMAIL_ADDRESS:="email@domain"}

check_openssl_installed
generate_certificate
