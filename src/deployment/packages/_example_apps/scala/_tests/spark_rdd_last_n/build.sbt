name := "kafka-streams-stateful"

version := "1.0"

scalaVersion := "2.12.12"

resolvers += "Maven Central" at "https://repo1.maven.org/maven2"
resolvers += "sbt Plugin Releases" at "https://repo.scala-sbt.org/scalasbt/sbt-plugin-releases"
resolvers += "Typesafe Releases" at "https://repo.typesafe.com/typesafe/ivy-releases"

libraryDependencies ++= Seq(
  "org.apache.spark" %% "spark-streaming-kafka-0-10" % "3.3.1",
  "org.apache.spark" %% "spark-sql-kafka-0-10" % "3.3.1" % Test,
  "org.apache.kafka" % "kafka-clients" % "3.3.1",
  "org.apache.spark" %% "spark-streaming" % "3.3.1" % "provided",
  "org.apache.spark" %% "spark-sql" % "3.3.1" % "provided",
  "org.scala-lang.modules" %% "scala-parser-combinators" % "2.1.1"
)

scalaSource in Compile := baseDirectory.value / "src" / "main" / "scala"

mainClass in Compile := Some("KafkaStatefulStream")

target in assembly := file("lib")

assemblyMergeStrategy in assembly := {
  case PathList("META-INF", xs @ _*) => MergeStrategy.discard
  case x                             => MergeStrategy.first
}
