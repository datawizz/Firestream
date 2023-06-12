
# Devcontainer

The devcontainer is a single containerized Debian instance which incorporates all setup/teardown/CI+CD
tools into a single image defined using infrustructure-as-code (Dockerfile + scripts). This very same container is used to
run all tests (using pytest) both locally and as a result of a commit (GitHub actions).

This container include Docker engine (moby) allowing all code to be run individually using docker-compose
It also contains Minikube (Kubernetes) which uses Docker engine to provide a full Kubernetes environment.

Because this is a container, is can be deployed:

1. In a browser using GitHub Codespaces for mobile development
2. On GitHub (Azure) as a GitHub action for tests
3. On a Mac (x86, or slower on M1?) using Docker Desktop for local development
4. #TODO on a Windows machine with WSL2 via SSH
5. On any debian linux host accessible by SSH, optionally using Visual Studio Code via SSH + Remote Container
6. On any public cloud on a debian linux VM
7. #TODO On Google Kubernetes Engine (GK8)

## Windows

Docker Desktop must be installed in Windows. This program does not depend on Docker Desktop, but the Firewall rules are set by the application in Windows
and simply installed the software is the easiest way to make it work.

# Credentials

The devcontainer expects either a .env file in this direcory.
Or a environment variable GOOGLE_DEFAULT_APPLICATION_CREDENTIALS
Or a CODESPACE_SECRET to be set.

## Codespaces

Codespaces allows for access to run the full project billed by the minute in a pay as you use model
directly from GitHub requiring only a browser. Thus it is possible for iPad remote development over cellular :)
Codespaces defines secrets.

## Docker Compose and testing

Taurus aims to be test driven and therefore testing aims for full coverage.
Tests are run using pytest. Services are run using docker compose up / down.

## Kubernetes

Kubernetes turns containerization to 11. <https://containerjournal.com/topics/container-ecosystems/kubernetes-vs-docker-a-primer/>
This is what should be used in production.

# TODO Tests are then run on the equivalent in Minikube using Kompose to build helm charts from docker compose files

## Windows

# TODO Add windows support (WSL2 should work by SSH port forwarding is wack. <https://www.youtube.com/watch?v=XkLjxr9iQ-8>)

## Toolkit

Used in

* CI/CD
* Development
* Build local
* build cloud
* bastion

### Sources

<https://github.com/microsoft/vscode-dev-containers>

<https://vsupalov.com/docker-arg-env-variable-guide/#the-dot-env-file-env>

<https://dev.to/bowmanjd/install-docker-on-windows-wsl-without-docker-desktop-34m9>

<https://medium.com/trabe/using-docker-engine-api-securely-584e0882158e>

<https://medium.com/@caysever/docker-compose-network-b86e424fad82>

<https://tomgregory.com/running-docker-in-docker-on-windows/>
