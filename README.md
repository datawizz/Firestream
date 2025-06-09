# Firestream

<picture width="500">
  <img
    src="assets/images/firestream_banner.png"
    alt="Firestream Logo"
  />
</picture>

## The Data Warehouse That Runs Anywhere

Firestream is a complete data warehouse that runs on your laptop and deploys seamlessly to the cloud. Think of it as "create-react-app" for data engineering but with more batteries included.

Firestream is composed of services from [Bitnami Helm Charts](https://github.com/bitnami/charts) including:

- ✅ Apache Spark via the [Spark Operator]([https://github.com/kubeflow/spark-operator) by Kubeflow
- ✅ Apache Kafka for streaming source / sink
- ✅ Apache Airflow for orchestration of Spark and other ETL pipelines
- ✅ MinIO for S3-compatible storage locally
- ✅ PostgreSQL for deployment metadata
- [ ] TODO Project Nessie for Git Ops on Data
- [ ] TODO Apache Iceberg for standardized Table abstraction
- [ ] TODO More Templates!

Firesteam uses template rendering to create Apache Spark Apps in Scala and Python. These rendered templates cross reference the environment driven configuration to that everything points to coherent source and target datasets.

### TLDR

**Problem**: Setting up a modern data stack requires configuring and integrating dozens of tools. Even "simple" tasks like ingesting streaming data or setting up a Spark cluster can take days of configuration.

**Solution**: Firestream provides a pre-configured, deployment-ready data stack that works out of the box with a single command.

## Why Firestream?

- **Zero to Data Warehouse in 5 Minutes**: No more spending days configuring Kafka, Spark, Airflow, and storage systems
- **Truly Portable**: Develop on your laptop, deploy to any cloud without changing a single line of code
- **Batteries Included**: Everything you need for modern data engineering pre-configured and ready
- **100% Open Source**: Built entirely on proven open-source technologies with business compatible licenses like Apache 2.0 and MIT.

## Quick Start

The only prerequisite is `docker` available on the `PATH` with a MacOS or Linux host.

```bash
git clone https://github.com/datawizz/Firestream.git && \
cd Firestream && \
bash docker/firestream/docker_preinit.sh && \
docker compose -f docker/firestream/docker-compose.devcontainer.yml exec devcontainer bash -l -c "cd /workspace && cargo run"
```

You can configure Firestream from the Terminal User Interface.

<picture width="500">
  <img
    src="assets/images/firestream-tui.gif"
    alt="Firestream Logo"
  />
</picture>




## Key Features

### **Instant Deployment**
Deploy a complete data warehouse faster than you can brew coffee. No DevOps expertise required.

### **Deployment-Ready Components**
- **Stream Processing**: Kafka + Spark Structured Streaming
- **Batch Processing**: Spark with automatic job submission
- **Orchestration**: Airflow with pre-configured DAGs
- **Storage**: S3-compatible MinIO with automatic partitioning

### **True Portability**
Develop locally, deploy anywhere:
- Local development on Docker/Podman
- One-command deployment to AWS, GCP, or Azure
- Consistent environment across all platforms

### **Extensible Architecture**
- Write ETL jobs in Python using simple classes
- Have Airflow orchastrate the execution of ETL Jobs
- Scale up your cluster as your needs grow
- Change out Minio for AWS S3 by changing the environment varibles

## How It Compares

The best way to understand Firestream is to compare it to the landscape of Data Tools and Cloud Providers

| Platform | Deployment Options | Local Development | Auto-Scaling | Custom ETL | Streaming Input | Streaming Output | Open Source | Identity & Access | Interface | Kubernetes Support | BI Integration |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **Firestream** | Any VM, Any Cloud | Yes | Yes | Yes | Webhooks, WebSocket, REST | Webhooks, WebSocket, REST | 100% | Coming Soon | CLI, VSCode, Web UI | Native | Apache Hive 2.0 API |
| **Snowflake** | Cloud Only | No | Yes | Yes | No | No | Proprietary | Yes | Web UI | No | Industry Standard |
| **Databricks** | Cloud Only | No | Yes | Yes | Limited | No | 90% (Apache Spark) | Yes | Web UI, Notebooks | No | Industry Standard |
| **Amazon Redshift** | AWS Only | No | Yes | Yes | No | No | ~90% (PostgreSQL) | Yes | Web UI | No | Industry Standard |
| **Google BigQuery** | GCP Only | No | Yes | Yes | No | No | Proprietary | Yes | Web UI | No | Industry Standard |
| **Azure Synapse** | Azure Only | No | Yes | Yes | No | No | Proprietary | Yes | Web UI | No | Multiple Standards |
| **AWS** | AWS Infrastructure | No | Yes | Yes | Via Kinesis/MSK | Via Kinesis/MSK | Mixed | Yes | Web Console | EKS Service | N/A (Platform) |
| **GCP** | GCP Infrastructure | No | Yes | Yes | Via Pub/Sub/Dataflow | Via Pub/Sub/Dataflow | Mixed | Yes | Web Console | GKE Service | N/A (Platform) |

## Architecture Overview

<picture>
  <img src="assets/images/stack.png" alt="Firestream Architecture" />
</picture>


All services are pre-configured to work together seamlessly.

## License

Firestream is licensed under the MIT License. See [LICENSE](LICENSE) for details.

Firestream is alpha software. Expect bugs and broken workflows while the API stabalizes.
---

Built by data engineers, for data engineers.
