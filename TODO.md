

https://www.traceable.ai/blog-post/scaling-the-kafka-consumer-for-a-machine-learning-service-in-python

performance testing https://stackoverflow.com/questions/62778701/getting-the-current-timestamp-of-each-row-of-dataframe-using-spark-java

Kafka Streams Benchmarking https://gist.github.com/jkreps/c7ddb4041ef62a900e6c

Implement flatMapGroupsWithState in Scala Spark API for transformation of generated text.





1. Update the Dockerfiles to build sbt assembly and run the Jar using spark submit and of java - jar

2. Consolidate Spark metronome and Spark prism into a single pyspark job which does both functions. No need for them to be seperate since the focus of the project is to display the different ways in which stream processing can be done.

3. Find a way to reliably get the timestamp of each record being processed at each stage that it is processed.



Setup full K8 cluster monitoring using Prometheus Operator
https://github.com/prometheus-operator/prometheus-operator
https://www.youtube.com/watch?v=_NtRkBipepg

# Install Prometheus
# helm repo add prometheus-community https://prometheus-community.github.io/helm-charts

# helm install prometheus prometheus-community/prometheus
#prometheus-server.default.svc.cluster.local

