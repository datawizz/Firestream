#!/usr/bin/env bash
#-------------------------------------------------------------------------------------------------------------
# Copyright (c) Justin Pottenger. All rights reserved.
#-------------------------------------------------------------------------------------------------------------
#

# Install KinD (Kubernetes in Docker)
install_kind() {
    KIND_VERSION="v0.12.0"
    # https://kind.sigs.k8s.io/
    curl -Lo ./kind https://kind.sigs.k8s.io/dl/${KIND_VERSION}/kind-linux-amd64

    chmod +x ./kind

    mv ./kind /usr/local/bin
}

install_kind

