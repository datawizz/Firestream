<!--
License
<!-- START Firestream -->
# Firestream
[![License](https://img.shields.io/:license-Apache%202-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0.txt)
[![GitHub Build](https://github.com/apache/airflow/workflows/Tests/badge.svg)](https://github.com/apache/airflow/actions)
[![Coverage Status](https://codecov.io/gh/apache/airflow/graph/badge.svg?token=WdLKlKHOAU)](https://codecov.io/gh/apache/airflow)
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)


<picture width="500">
  <img
    src="static/images/firestream_banner.png"
    alt="Firestream Logo"
  />
</picture>

Firestream is a serverless data warehouse designed to fill the gaps left by the use-case-specific point solutions that make up the average data engineer's toolbelt.

Inspired by OpenStack, Firestream is a easy to deploy fully local K3S cluster than can easily be shipped to production in GCP or AWS. Think of it as the "create-react-app" for data warehousing, evolving into a comprehensive stack tailored for Data Scientists by Data Engineers.

Setups differ widely among machines and environments. Even modern solutions like Bytewax require an external kubernetes setup to run their own workflows. But what if you need to deploy a new API to intake data? Need to setup Airflow? Spark Cluster? Minio? A new internal microtool with that one library? In a secure environment that is accessible based on RBAC? What about the Kitchen Sink? Firestream will generated kubernetes deployments for you! (provided you adopt the config, RBAC TBD)

Firestream adopts the Dataflow paradigm, ensuring data is touched minimally throughout its lifecycle. Acknowledging that data has gravity and prefers to stay in place, Firestream emphasizes the necessity of highly specific ETL processes required for modern data meshes, namely receiving streaming data and processing it.

The name "Firestream" combines the idea of "firing up" an instant data solution with "stream" representing the continuous data flow, mirroring how the tool provides quick deployment of data streaming infrastructure.

## Key Features
* ETL Configuration: Customize your ETL jobs in Python using classes to represent whole apps.

* Reverse Compatibility: Seamlessly integrates with Apache Airflow as the orchestrator, ensuring compatibility with existing workflows.

* Production-Ready Environment: A robust development environment for both stateful and stateless ETL and streaming dashboarding, ready for production deployment with minimal configuration.



# Comps

The best way to understand Firestream is to compare it to the landscape of Data Tools and Cloud Providers

| Tool/Platform            | Cloud Support                  | Runs on a Laptop | Dynamic Scalability | Code your own ETL | Real-Time Streaming Source | Real-Time Streaming Effects | Built on Open Source | Identity Management | User-Friendly Interface         | Kubernetes as a Service | BI Tool Compatible                    |
|--------------------------|--------------------------------|------------------|---------------------|-------------------|----------------------------|----------------------------|-----------------------|----------------------|----------------------------------|------------------------|---------------------------------------|
| Firestream               | Yes, deploy to arbitrary VM(s) | Yes              | Yes                 | Yes               | Webhooks, Websocket, REST  | Webhooks, Websocket, REST  | 100%                  | #TODO                | CLI, IDE (vscode), Services Interfaces | Yes                    | Apache Hive 2.0 API w integrated Catalog |
| Amazon Redshift          | Yes                            | No               | Yes                 | Yes               |                            | No                         | ~90%, postgres        | Yes                  | GUI                              | No                     | Widely Supported                      |
| Google BigQuery          | Yes                            | No               | Yes                 | Yes               |                            | No                         | 0%                    | Yes                  | Yes                              | No                     | Widely Supported                      |
| Microsoft Azure Synapse  | Yes                            | No               | Yes                 | Yes               |                            | No                         | 0%                    | Yes                  | Yes                              | No                     | ?                                     |
| Snowflake                | Yes                            | No               | Yes                 | Yes               |                            | No                         | 0%                    | Yes                  | Yes                              | No                     | Widely Supported                      |
| Databricks               | Yes                            | No               | Yes                 | Yes               |                            | No                         | 90%, spark            | Yes                  | Yes                              | No                     | Widely Supported                      |
| Google Cloud Platform (GCP) | Yes                         | No               | Yes                 | Yes               |                            | With dedicated project     | Extensive             | Yes                  | Yes                              | Yes                    | N/A                                   |
| Amazon Web Services (AWS) | Yes                           | No               | Yes                 | Yes               |                            | With dedicated project     | Extensive             | Yes                  | Yes                              | Yes                    | N/A                                   |






# Table of Contents

[Tech Stack](#tech-stack)
[Getting Started](#getting-started)
[Development Container](#development-container)
[Apache Spark Structured Streaming](#apache-spark-structured-streaming)
[Python Stateful Streaming](#python-stateful-streaming)
[Node.js Middleware](#node-middleware)
[Plotly.js Dashboard](#plotly-dashboard)

# Tech Stack

Firestream is powered by these core technologies.


* [Development Container](https://github.com/devcontainers)
* ["Docker-From-Docker"](https://github.com/devcontainers/features/tree/main/src/docker-outside-of-docker)
* [k3s](https://k3s.io/)
* [k3d](https://github.com/k3d-io/k3d)
* [Helm](https://helm.sh/)
* [Bitnami Charts](https://github.com/bitnami/charts)
  * [Minio](https://github.com/bitnami/charts/tree/main/bitnami/minio)
  * [Kafka](https://github.com/bitnami/charts/tree/main/bitnami/kafka)
  * [Postgres](https://github.com/bitnami/charts/tree/main/bitnami/postgresql)
  * [Contour (Envoy)](https://github.com/bitnami/charts/tree/main/bitnami/contour)
* [Spark via Spark Operator](https://github.com/kubeflow/spark-operator)



# Getting Started

This project requires `containerd` and a x86 / AMD64 / Arm64 linux host.

It has been tested on MacOS with Docker Desktop and Ubuntu using Podman.

This project implements **Infrustructure as Code** via a Devcontainer (Development Container) defined in a Dockerfile. The project can be run using the following command:

```

git clone https://github.com/datawizz/firestream.git && cd firestream && python bootstrap.py

```

This will use the Docker Engine of the host and bind to the var/run/docker.sock to create the Devcontainer, open it via a terminal, and bootstrap the project. You will be guided through configuration of the environment.

Alternatively you can pass a JSON file describing the desired deployment like so.


```
python bootstrap.py path/to/json/config.json
```

Example minimal config

```json
{
  "services": [
    {
      "service_name": "airflow",
      "resources": {
        "requests": {
          "cpu": "200m",
          "memory": "128Mi"
        },
      },

    {
      "service_name": "kafka",
      "resources": {
        "requests": {
          "cpu": "200m",
          "memory": "128Mi"
        },
      },
    },
    "service_name": "bigquery",
      "resources": {
        "requests": {
          "cpu": "200m",
          "memory": "128Mi"
        },
      },
  ],
  "image": {
    "repository": "python:",
    "tag": "3.9-slim",
    "pullPolicy": "IfNotPresent"
  },
  "service": {
    "ports": [80,443]
  },
  "replicaCount": 1,
  "resources": {
    "requests": {
      "cpu": "200m",
      "memory": "128Mi"
    },
  },
  "ingress": {
    "enabled": false,
    "external": false
  }
}

```

# Development Container

Alternatively you can run this project using the VS Code Devcontainer extension. Simply clone the repo, open it in VS Code, and click "open in container" on the bottom left.

The Devcontainer is configured to use IP Tables to resolve Kubernetes internal services using CoreDNS hosted in the Kind Control Plane using the Docker Engine as a bridge on the host's network. This allows anything run within the Devcontainer to reach local Kubernetes services using the same URL as it would inside the Kubernetes cluster!

i.e.

```
ping service_name.namespace.svc.cluster.local
```

Is resolvable in the Devcontainer.

This is basically a blanket kubectl proxy command but since it is run within the Devcontainer (and Docker Engine) it is agnostic to the underlying operating system's networking approach. Further this Devcontainer can be used with minimal configuration to forward kubectl commands to any cloud provider (AWS, GCP, etc).

The **Kubernetes** cluster is run using KinD (Kubernetes in Docker).

![Screenshot](static/images/stack.png)


### Apache Spark Structured Streaming

Data is generated using a PySpark application which sends data to the "metronome" topic in Kafka. This application uses the Spark Rate-Micro-Batch (new in Spark 3.3) to generate exactly 1 record per second advanced. This simulates a monotonically increasing measurement from some sensor.

This stream is joined to a small table to create multiple sensor readings on the same update frequency. The field DeviceID is used to partition the output before writing to Kafka.

A second Spark application in Scala reads from the topic "metronome" and performs Stateful Stream Processing in which the N-1 record is compared with the Nth record to find the next advance in the Wiener process. Scala is used here since the "MapGroupsWithState" function is currently only available using the Scala API.

### Python Stateful Streaming

For a comparison of performance a Python application is configured to read data from the "metronome" topic in Kafka. Similar to the Spark Scala application the goal is to merge the N-1 and Nth events in the stream coming from a Kafka topic to demonstrate a Wiener process.

### Node Middleware

Kafka is great for internal communication between microservices but it is not ideal for clients to connect to directly due to the need to secure it and scale it 1-1 with the clients that are connected. Instead a middleware service is implemented which reads the events from Kafka and publishes the events one by one to all connected websocket clients. Events from Kakfa are read and added to a queue. Asynchronously the queue is read and sent to all connected websocket clients.

### Plotly Dashboard

Plotly is a powerful library for data visualization and provides bindings in Python to make a Data Scientist's life easier. But in the realm of quickly updating dashboards the Python layer is simply too slow.

Implemented here is the Plotly JS library rendered client side using NextJS and React. The single page application subscribes to the Websocket connection provided by the middleware and updates events every 100 milliseconds.

The dashboard is available at localhost:8000

![Screenshot](images/dashboard.gif)
