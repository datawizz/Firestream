name := "spark-scala-udf-streaming-lag"

version := "0.0.1-SNAPSHOT"
scalaVersion := "2.12"

libraryDependencies ++= Seq(
  "org.apache.spark" %% "spark-sql" % "3.3.1" % "provided"
)