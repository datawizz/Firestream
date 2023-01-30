

![Screenshot](images/fireworks_banner.png)


Fireworks is a minimal development environment for denomstration of streaming ETL, Feature Generation, and Real Time Dashboarding. It uses the following stack:

* Development Container (a bastion container with modified DNS)
* Docker-From-Docker access to underlying Docker Engine
* KinD (Kubernetes in Docker)
* Helm
* Kafka (Bitnami)
* Spark (Bitnami)
* Spark Structured Streaming
* Python Kafka Consumer / Producer (librdkafka)
* Plotly Dash

## Getting Started

This project requires Docker and a x86/AMD64 Debian/Ubuntu environment. This project has been tested on Windows WSL Ubuntu 20.04 and Ubuntu 20.04 on bare metal.

This project implements **Infrustructure as Code** via a Devcontainer (Development Container) defined in a Dockerfile. The project can be run using the following command:

```
git clone repo && cd k8-spark-kafka && sh start.sh
```

This will use the Docker Engine of the host and bind to the var/run/docker.sock to create the Devcontainer, open it via a terminal, and bootstrap the project. Once everything is built it will then forward the dashboard to localhost:3000.

Alternatively you can run this project using the VS Code Devcontainer extension. Simply clone the repo, open it in VS Code, and click "open in container" on the bottom left.

## Networking

The Devcontainer is configured to use IP Tables to resolve Kubernetes internal services using CoreDNS hosted in the Kind Control Plane using the Docker Engine as a bridge on the host's network. This allows anything run within the Devcontainer to reach local Kubernetes services using the same URL as it would inside the Kubernetes cluster!

i.e. 
```
ping service_name.namespace.svc.cluster.local
```
Is resolvable in the Devcontainer.

This is basically a blanket kubectl proxy command but since it is run within the Devcontainer (and Docker Engine) it is agnostic to the underlying operating system's networking approach. Further this Devcontainer can be used with minimal configuration to forward kubectl commands to any cloud provider (AWS, GCP, etc).

The **Kubernetes** cluster is run using KinD (Kubernetes in Docker).

![Screenshot](images/stack.png)

## Streaming ETL & Stateful Feature Generation

To demonstrate **Streaming ETL** Apache Spark and Apache Kafka are run using Bitnami Helm Charts for minimal configuration and convenience. The Confluent Python library is used to manipulate data retrived from and then written to Kafka. Plotly Dash is used to plot streaming data from Kafka.


![Screenshot](images/pipeline.png)



### Spark Application

To demontrate a **Spark Application** the first stage in the pipeline uses the Spark Rate-Micro-Batch (new in Spark 3.3) to generate exactly 1 record per second. This simulates a monotonically increasing measurement from some sensor.

This stream is joined to a small table to create multiple sensor readings on the same update frequency. The field DeviceID is used to partition the output before writing to Kafka.

### Python Brownian Motion

To demonstrate **stateful processing of a stream** a Python script is used to merge the N and N-1 events in the event stream coming from a Kafka topic over multiple partitions. The last 100 records sorted by time for each key are kept in memory and a Markov Chain is created for each DeviceID before writing to Kafka.

This is analogous to the lag(1) function provided in Spark SQL which is not available as of 3.3 in streaming mode. Another alternative would be flatMapGroupsWithState from the Scala API which could manage state better for production scenarios.

### Dashboard

To demonstrate **Data Visualization** a Python iterator over a Kafka topic is passed to Plotly Dash callback. Multithreading is used for concurrency in read, queue, and serve. This is packaged in a Dockerfile which is deployed as **Microservice** in the K8 cluster.

The dashboard is available at localhost:8080

![Screenshot](images/dashboard.gif)