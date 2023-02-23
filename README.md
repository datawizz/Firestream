

![Screenshot](images/fireworks_banner.png)


Fireworks aims to be a production ready development environment for streaming stateful/stateless ETL and streaming Dashboarding. The purpose of Fireworks is to provide a minimal implementation of a modern streaming Data Warehouse using popular technologies that can be deployed to a production kubernetes cluster (such as Amazon EKS or Google GKE) with minimal additional configuration.

# Table of Contents  
[Tech Stack](#tech-stack)  
[Getting Started](#getting-started)  
[Development Container](#development-container)  
[Apache Spark Structured Streaming](#apache-spark-structured-streaming)  
[Python Stateful Streaming](#python-stateful-streaming)  
[Node.js Middleware](#node-middleware)  
[Plotly.js Dashboard](#plotly-dashboard)  

# Tech Stack

Fireworks makes use of the following technologies.

* Development Container
* Docker-From-Docker
* KinD (Kubernetes in Docker)
* Helm
* Kafka (Bitnami)
* Spark (Bitnami)
* Python Kafka Consumer / Producer (librdkafka)
* Node.js Middlware Websocket proxy
* Plotly.js Dash (Next.JS, React)

# Getting Started

This project requires Docker and a x86/AMD64 Debian/Ubuntu environment. This project has been tested on Windows WSL Ubuntu 20.04 and Ubuntu 20.04 on bare metal.

This project implements **Infrustructure as Code** via a Devcontainer (Development Container) defined in a Dockerfile. The project can be run using the following command:

```
git clone https://github.com/datawizz/fireworks.git && cd fireworks && sh bootstrap.sh
```

This will use the Docker Engine of the host and bind to the var/run/docker.sock to create the Devcontainer, open it via a terminal, and bootstrap the project. Once everything is built it will then expose the dashboard on localhost:3000.

Alternatively you can run this project using the VS Code Devcontainer extension. Simply clone the repo, open it in VS Code, and click "open in container" on the bottom left.

# Development Container

The Devcontainer is configured to use IP Tables to resolve Kubernetes internal services using CoreDNS hosted in the Kind Control Plane using the Docker Engine as a bridge on the host's network. This allows anything run within the Devcontainer to reach local Kubernetes services using the same URL as it would inside the Kubernetes cluster!

i.e. 
```
ping service_name.namespace.svc.cluster.local
```
Is resolvable in the Devcontainer.

This is basically a blanket kubectl proxy command but since it is run within the Devcontainer (and Docker Engine) it is agnostic to the underlying operating system's networking approach. Further this Devcontainer can be used with minimal configuration to forward kubectl commands to any cloud provider (AWS, GCP, etc).

The **Kubernetes** cluster is run using KinD (Kubernetes in Docker).

![Screenshot](images/stack.png)


### Apache Spark Structured Streaming

Data is generated using a PySpark application which sends data to the "metronome" topic in Kafka. This application uses the Spark Rate-Micro-Batch (new in Spark 3.3) to generate exactly 1 record per second advanced. This simulates a monotonically increasing measurement from some sensor.

This stream is joined to a small table to create multiple sensor readings on the same update frequency. The field DeviceID is used to partition the output before writing to Kafka.

A second Spark application in Scala reads from the topic "metronome" and performs Stateful Stream Processing in which the N-1 record is compared with the Nth record to find the next advance in the Wiener process. Scala is used here since the "MapGroupsWithState" function is currently only available using the Scala API.

### Python Stateful Streaming

For a comparison of performance a Python application is configured to read data from the "metronome" topic in Kafka. Similar to the Spark Scala application the goal is to merge the N-1 and Nth events in the stream coming from a Kafka topic to demonstrate a Wiener process.

### Node Middleware

Kafka is great for internal communication between microservices but it is not ideal for clients to connect to directly due to the need to secure it and scale it with the number of clients that are connected. Instead a middleware service is implemented in Node.js which reads the events from Kafka and publishes the most recent events to all connected websocket clients. Events from Kakfa are read and added to a queue for the life of the pod. Asynchronously the queue is read and sent to all connected websocket clients.

The Middleware is also responsible for sending a ping-pong request every 5 seconds to each connected client ensuring they are still there and listening. This service also handles client disconnection and reconnection ensuring a smooth experience.

### Plotly Dashboard

Plotly is a powerful library for data visualization and provides bindings in Python to make a Data Scientist's life easier. But in the realm of quickly updating dashboards the Python layer is simply too slow.

Implemented here is the Plotly JS library rendered client side using NextJS and React. The somewhat heavy single page application subscribes to the Websocket connection provided by the middleware and updates events every 100 milliseconds.

The dashboard is available at localhost:3000


![Screenshot](images/dashboard.gif)